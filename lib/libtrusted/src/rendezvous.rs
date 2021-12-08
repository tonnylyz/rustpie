//! A rendezvous-based channel for synchronous Inter-Task Communication (ITC).
//!
//! This crate offers a rendezvous channel, in which two tasks can exchange messages
//! without an intermediary buffer.
//! The sender and receiver tasks must rendezvous together to exchange data,
//! so at least one of them must block.
//!
//! Only `Send` types can be sent or received through the channel.
//!
//! This is not a zero-copy channel;
//! To avoid copying large messages, use a reference (layer of indirection) like `Box`.
//!
//! TODO: add support for a queue of pending senders and receivers
//!       so that we can enable MPMC (multi-producer multi-consumer) behavior
//!       that allows senders and receivers to be cloned.
//!       Note that currently only a single receiver and single sender is supported.

use core::fmt;
use core::ptr;
use alloc::sync::Arc;
use spin::Mutex;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TaskRef(usize); // hold tid
impl TaskRef {
  fn block(&self) {
    // microcall::thread_set_status(self.0, common::thread::THREAD_STATUS_NOT_RUNNABLE);
  }
  fn unblock(&self) {
    // microcall::thread_set_status(self.0, common::thread::THREAD_STATUS_RUNNABLE);
  }
}

mod wait_queue {
  extern crate alloc;
  use alloc::collections::VecDeque;
  use spin::Mutex;
  use super::TaskRef;

  /// An object that holds a blocked `Task`
  /// that will be automatically unblocked upon drop.
  pub struct WaitGuard {
    task: TaskRef,
  }
  impl WaitGuard {
    /// Blocks the given `Task` and returns a new `WaitGuard` object
    /// that will automatically unblock that Task when it is dropped.
    pub fn new(task: TaskRef) -> WaitGuard {
      task.block();
      WaitGuard {
        task: task,
      }
    }

    /// Blocks the task guarded by this waitguard,
    /// which is useful to re-block a task after it spuriously woke up.
    pub fn block_again(&self) {
      self.task.block();
    }

    /// Returns a reference to the `Task` being blocked in this `WaitGuard`.
    pub fn task(&self) -> &TaskRef {
      &self.task
    }
  }
  impl Drop for WaitGuard {
    fn drop(&mut self) {
      self.task.unblock();
    }
  }


  /// Errors that may occur while waiting on a waitqueue/condition/event.
  #[derive(Debug, PartialEq)]
  pub enum WaitError {
    NoCurrentTask,
    Interrupted,
    Timeout,
    SpuriousWakeup,
  }

  /// A queue in which multiple `Task`s can wait for other `Task`s to notify them.
  ///
  /// This can be shared across multiple `Task`s by wrapping it in an `Arc`.
  pub struct WaitQueue(Mutex<VecDeque<TaskRef>>);

// ******************************************************************
// ************ IMPORTANT IMPLEMENTATION NOTE ***********************
// All modification of task runstates must be performed atomically
// with respect to adding or removing those tasks to/from the waitqueue itself.
// Otherwise, there could be interleavings that result in tasks not being notified properly,
// or not actually being put to sleep when being placed on the waitqueue,
// or the task being switched away from after setting itself to blocked (when waiting)
// but before it can release its lock on the waitqueue.
//    (Because once a task is blocked, it can never run again and thus
//     has no chance to release its waitqueue lock, causing deadlock).
// Thus, we disable preemption (well, currently we disable interrupts)
// AND hold the waitqueue lock while changing task runstate,
// which ensures that once the task is blocked it will always release its waitqueue lock.
// ******************************************************************

  impl WaitQueue {
    /// Create a new empty WaitQueue.
    pub fn new() -> WaitQueue {
      WaitQueue::with_capacity(4)
    }

    /// Create a new empty WaitQueue.
    pub fn with_capacity(initial_capacity: usize) -> WaitQueue {
      WaitQueue(Mutex::new(VecDeque::with_capacity(initial_capacity)))
    }

    /// Puts the current `Task` to sleep where it blocks on this `WaitQueue`
    /// until it is notified by another `Task`.
    ///
    /// If the `Task` wakes up spuriously (it is still on the waitqueue),
    /// it will be automatically put back to sleep until it is properly woken up.
    /// Therefore, there is no need for the caller to check for spurious wakeups.
    ///
    /// This function blocks until the `Task` is woken up through the notify mechanism.
    pub fn wait(&self) -> Result<(), WaitError> {
      self.wait_until(&|/* _ */| Some(()))
    }

    /// Similar to [`wait`](#method.wait), but this function blocks until the given
    /// `condition` closure returns `Some(value)`, and then returns that `value` inside `Ok()`.
    ///
    /// The `condition` will be executed atomically with respect to the wait queue,
    /// which avoids the problem of a waiting task missing a "notify" from another task
    /// due to interleaving of instructions that may occur if the `condition` is checked
    /// when the wait queue lock is not held.
    ///
    // /// The `condition` closure is invoked with one argument, an immutable reference to the waitqueue,
    // /// to allow the closure to examine the condition of the waitqueue if necessary.
    pub fn wait_until<R>(&self, condition: &dyn Fn(/* &VecDeque<TaskRef> */) -> Option<R>) -> Result<R, WaitError> {
      let curr_task = TaskRef(microcall::get_tid());

      // Do the following atomically:
      // (1) Obtain the waitqueue lock
      // (2) Add the current task to the waitqueue
      // (3) Set the current task's runstate to `Blocked`
      // (4) Release the lock on the waitqueue.
      loop {
        {
          let mut wq_locked = self.0.lock();
          if let Some(ret) = condition(/* &wq_locked */) {
            return Ok(ret);
          }
          // This is only necessary because we're using a non-Set waitqueue collection that allows duplicates
          if !wq_locked.contains(&curr_task) {
            wq_locked.push_back(curr_task.clone());
          } else {
            warn!("WaitQueue::wait_until():  task was already on waitqueue (potential spurious wakeup?). {:?}", curr_task);
          }
          // trace!("WaitQueue::wait_until():  putting task to sleep: {:?}\n    --> WQ: {:?}", curr_task, &*wq_locked);
          curr_task.block();
        }
        // scheduler::schedule();
        microcall::thread_yield();

        // Here, we have been woken up, so loop back around and check the condition again
        // trace!("WaitQueue::wait_until():  woke up!");
      }
    }

    /// Similar to [`wait_until`](#method.wait_until), but this function accepts a `condition` closure
    /// that can mutate its environment (a `FnMut`).
    pub fn wait_until_mut<R>(&self, condition: &mut dyn FnMut(/* &VecDeque<TaskRef> */) -> Option<R>) -> Result<R, WaitError> {
      let curr_task = TaskRef(microcall::get_tid());

      // Do the following atomically:
      // (1) Obtain the waitqueue lock
      // (2) Add the current task to the waitqueue
      // (3) Set the current task's runstate to `Blocked`
      // (4) Release the lock on the waitqueue.
      loop {
        {
          let mut wq_locked = self.0.lock();
          if let Some(ret) = condition(/* &wq_locked */) {
            return Ok(ret);
          }
          // This is only necessary because we're using a non-Set waitqueue collection that allows duplicates
          if !wq_locked.contains(&curr_task) {
            wq_locked.push_back(curr_task.clone());
          } else {
            warn!("WaitQueue::wait_until():  task was already on waitqueue (potential spurious wakeup?). {:?}", curr_task);
          }
          // trace!("WaitQueue::wait_until():  putting task to sleep: {:?}\n    --> WQ: {:?}", curr_task, &*wq_locked);
          curr_task.block();
        }
        // scheduler::schedule();
        microcall::thread_yield();

        // Here, we have been woken up, so loop back around and check the condition again
        // trace!("WaitQueue::wait_until():  woke up!");
      }
    }

    /// Wake up one random `Task` that is waiting on this queue.
    /// # Return
    /// * returns `Ok(true)` if a `Task` was successfully woken up,
    /// * returns `Ok(false)` if there were no `Task`s waiting.
    pub fn notify_one(&self) -> bool {
      self.notify(None)
    }

    /// Wake up a specific `Task` that is waiting on this queue.
    /// # Return
    /// * returns `true` if the given `Task` was waiting and was woken up,
    /// * returns `false` if there was no such `Task` waiting.
    pub fn notify_specific(&self, task_to_wakeup: &TaskRef) -> bool {
      self.notify(Some(task_to_wakeup))
    }

    /// The internal routine for notifying / waking up tasks that are blocking on the waitqueue.
    /// If specified, the given `task_to_wakeup` will be notified,
    /// otherwise the first task on the waitqueue will be notified.
    fn notify(&self, task_to_wakeup: Option<&TaskRef>) -> bool {
      // trace!("  notify [top]: task_to_wakeup: {:?}", task_to_wakeup);

      // Do the following atomically:
      // (1) Obtain the waitqueue lock
      // (2) Choose a task and remove it from the waitqueue
      // (3) Set that task's runstate to `Runnable`
      // (4) Release the lock on the waitqueue.

      let mut wq_locked = self.0.lock();
      let tref = if let Some(ttw) = task_to_wakeup {
        // find a specific task to wake up
        let index = wq_locked.iter().position(|t| t == ttw);
        index.and_then(|i| wq_locked.remove(i))
      } else {
        // just wake up the first task
        wq_locked.pop_front()
      };

      // trace!("  notify: chose task to wakeup: {:?}", tref);
      if let Some(t) = tref {
        // trace!("WaitQueue::notify():  unblocked task on waitqueue\n    --> WQ: {:?}", &*wq_locked);
        t.unblock();
        true
      } else {
        // trace!("WaitQueue::notify():  did nothing");
        false
      }
    }
  }
}

use wait_queue::{WaitQueue, WaitGuard, WaitError};

// For Minix Fault injections
struct FaultNum{
  fault_num : usize
}

static FAULT_ITEM: Mutex<FaultNum> = Mutex::new(FaultNum{fault_num : 0});

pub fn set_fault_item (i : usize) {
  let mut e = FAULT_ITEM.lock();
  e.fault_num = i;
}

pub fn get_fault_item () -> usize {
  let e = FAULT_ITEM.lock();
  e.fault_num.clone()
}


/// A wrapper type for an `ExchangeSlot` that is used for sending only.
struct SenderSlot<T>(Arc<Mutex<ExchangeState<T>>>);
/// A wrapper type for an `ExchangeSlot` that is used for receiving only.
struct ReceiverSlot<T>(Arc<Mutex<ExchangeState<T>>>);


/// An `ExchangeSlot` consists of two references to a shared state
/// that is used to exchange a message.
///
/// There is a "sender" reference and a "receiver" reference,
/// which are wrapped in their respective types: `SenderSlot` and `ReceiverSlot`.
struct ExchangeSlot<T> {
  sender:   Mutex<Option<SenderSlot<T>>>,
  receiver: Mutex<Option<ReceiverSlot<T>>>,
}
impl<T> ExchangeSlot<T> {
  fn new() -> ExchangeSlot<T> {
    let inner = Arc::new(Mutex::new(ExchangeState::Init));
    ExchangeSlot {
      sender: Mutex::new(Some(SenderSlot(inner.clone()))),
      receiver: Mutex::new(Some(ReceiverSlot(inner))),
    }
  }

  fn take_sender_slot(&self) -> Option<SenderSlot<T>> {
    self.sender.lock().take()
  }

  fn take_receiver_slot(&self) -> Option<ReceiverSlot<T>> {
    self.receiver.lock().take()
  }

  fn replace_sender_slot(&self, s: SenderSlot<T>) {
    let _old = self.sender.lock().replace(s);
    if _old.is_some() {
      error!("BUG: REPLACE SENDER SLOT WAS SOME ALREADY");
    }
  }

  fn replace_receiver_slot(&self, r: ReceiverSlot<T>) {
    let _old = self.receiver.lock().replace(r);
    if _old.is_some() {
      error!("BUG: REPLACE RECEIVER SLOT WAS SOME ALREADY");
    }
  }
}


/// The possible states of an exchange slot in a rendezvous channel.
/// TODO: we should improve this state machine using session types
///       to check for valid state transitions at compile time.
enum ExchangeState<T> {
  /// Initial state: we're waiting for either a sender or a receiver.
  Init,
  /// A sender has arrived before a receiver.
  /// The `WaitGuard` contains the blocked sender task,
  /// and the `T` is the message that will be exchanged.
  WaitingForReceiver(WaitGuard, T),
  /// A receiver has arrived before a sender.
  /// The `WaitGuard` contains the blocked receiver task.
  WaitingForSender(WaitGuard),
  /// Sender and Receiver have rendezvoused, and the receiver finished first.
  /// Thus, it is the sender's responsibility to reset to the initial state.
  ReceiverFinishedFirst,
  /// Sender and Receiver have rendezvoused, and the sender finished first.
  /// Thus, the message `T` is enclosed here for the receiver to take,
  /// and it is the receivers's responsibility to reset to the initial state.
  SenderFinishedFirst(T),
}
impl<T> fmt::Debug for ExchangeState<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "ExchangeState::{}", match self {
      ExchangeState::Init                     => "Init",
      ExchangeState::WaitingForReceiver(..)   => "WaitingForReceiver",
      ExchangeState::WaitingForSender(..)     => "WaitingForSender",
      ExchangeState::ReceiverFinishedFirst    => "ReceiverFinishedFirst",
      ExchangeState::SenderFinishedFirst(..)  => "SenderFinishedFirst",
    })
  }
}

// enum RendezvousState<T> {
//     /// Initial state: we're waiting for either a sender or a receiver.
//     Init,
//     /// A task is blocked and waiting to rendezvous; the blocked task is held in the `WaitGuard`.
//     /// The `Option<T>` is for exchanging the message, and indicates whether the blocked task is a sender or receiver.
//     /// * If `None`, then the receiver is blocked, waiting on a sender to put its message into `Some(T)`.
//     /// * If `Some`, then the sender is blocked, waiting on a receiver to take the message out of `Option<T>`.
//     Waiting(WaitGuard, Option<T>),
// }


/// Create a new channel that requires a sender a receiver to rendezvous
/// in order to exchange a message.
///
/// Returns a tuple of `(Sender, Receiver)`.
pub fn new_channel<T: Send>() -> (Sender<T>, Receiver<T>) {
  let channel = Arc::new(Channel::<T> {
    slot: ExchangeSlot::new(),
    waiting_senders: WaitQueue::new(),
    waiting_receivers: WaitQueue::new(),
  });
  (
    Sender   { channel: channel.clone() },
    Receiver { channel: channel }
  )
}



/// The inner channel for synchronous rendezvous-based communication
/// between `Sender`s and `Receiver`s.
///
/// This struct contains one or more exchange slot(s) (`ExchangeSlot`) as well as
/// queues for tasks that waiting to send or receive messages via those exchange slots.
///
/// Sender-side and Receiver-side references to an exchange slot can be obtained in both
/// a blocking and non-blocking fashion,
/// which supports both synchronous (rendezvous-based) and asynchronous channels.
struct Channel<T: Send> {
  /// In a zero-capacity synchronous channel, there is only a single slot,
  /// but senders and receivers perform a blocking wait on it until the slot becomes available.
  /// In contrast, a synchronous channel with a capacity of 1 would return a "channel full" error
  /// if the slot was taken, instead of blocking.
  slot: ExchangeSlot<T>,
  waiting_senders: WaitQueue,
  waiting_receivers: WaitQueue,
}
impl<T: Send> Channel<T> {
  /// Obtain a sender slot, blocking until one is available.
  fn take_sender_slot(&self) -> Result<SenderSlot<T>, WaitError> {

    let fault_id = get_fault_item();

    if fault_id == 5 {
      set_fault_item(0);
      unsafe {
        let waiting_senders = &self.waiting_senders;
        let wq_ptr = &(waiting_senders) as *const _ as usize;
        let p1 = (wq_ptr) as *mut usize;
        ptr::write(p1,0xDEADBEEF);
        let res =waiting_senders.wait_until(&|| self.try_take_sender_slot());
        return res
      }
    }

    // Fast path: the uncontended case.
    if let Some(s) = self.try_take_sender_slot() {
      return Ok(s);
    }


    // Slow path: add ourselves to the waitqueue
    // info!("waiting to acquire sender slot...");
    let res = self.waiting_senders.wait_until(&|| self.try_take_sender_slot());
    // info!("... acquired sender slot!");
    res
  }

  /// Obtain a receiver slot, blocking until one is available.
  fn take_receiver_slot(&self) -> Result<ReceiverSlot<T>, WaitError> {
    let fault_id = get_fault_item();
    if fault_id == 10 {
      set_fault_item(0);
      return Err(WaitError::NoCurrentTask);
    }

    if fault_id == 9 {
      set_fault_item(0);
      unsafe {
        let waiting_receivers = &self.waiting_receivers;
        let wq_ptr = &(waiting_receivers) as *const _ as usize;
        let p1 = (wq_ptr) as *mut usize;
        ptr::write(p1,0xDEADBEEF);
        let res =waiting_receivers.wait_until(&|| self.try_take_receiver_slot());
        return res
      }
    }

    // Fast path: the uncontended case.
    if let Some(s) = self.try_take_receiver_slot() {
      return Ok(s);
    }

    // Slow path: add ourselves to the waitqueue
    // trace!("waiting to acquire receiver slot...");
    let res = self.waiting_receivers.wait_until(&|| self.try_take_receiver_slot());
    // trace!("... acquired receiver slot!");
    res
  }

  /// Try to obtain a sender slot in a non-blocking fashion,
  /// returning `None` if a slot is not immediately available.
  fn try_take_sender_slot(&self) -> Option<SenderSlot<T>> {
    self.slot.take_sender_slot()
  }

  /// Try to obtain a receiver slot in a non-blocking fashion,
  /// returning `None` if a slot is not immediately available.
  fn try_take_receiver_slot(&self) -> Option<ReceiverSlot<T>> {
    self.slot.take_receiver_slot()
  }
}


/// The sender (transmit) side of a channel.
#[derive(Clone)]
pub struct Sender<T: Send> {
  channel: Arc<Channel<T>>,
}
impl <T: Send> Sender<T> {
  /// Send a message, blocking until a receiver is ready.
  ///
  /// Returns `Ok(())` if the message was sent and received successfully,
  /// otherwise returns an error.
  pub fn send(&self, msg: T) -> Result<(), &'static str> {

    let fault_id = get_fault_item();
    if fault_id == 1 {
      set_fault_item(0);
      unsafe { *(0x5050DEADBEEF as *mut usize) = 0x5555_5555_5555; }
    }

    if fault_id == 2 {
      set_fault_item(0);
      unsafe {
        let msg_ptr = &(msg) as *const _ as usize;
        let p1 = (msg_ptr) as *mut usize;
        ptr::write(p1,0xDEADBEEF);
      }
    }

    if fault_id == 3 {
      set_fault_item(0);
      unsafe {
        let self_ptr = &(self) as *const _ as usize;
        let p1 = (self_ptr) as *mut usize;
        ptr::write(p1,0xDEADBEEF);
      }
    }

    if fault_id == 12 {
      set_fault_item(0);
      unsafe {
        let msg_ptr = &(msg) as *const _ as usize;
        let p1 = (msg_ptr) as *mut usize;
        ptr::write(p1,0x0);
      }
    }


    // trace!("rendezvous: send() entry");
    let curr_task = TaskRef(microcall::get_tid());

    // obtain a sender-side exchange slot, blocking if necessary
    let sender_slot = self.channel.take_sender_slot().map_err(|_| "failed to take_sender_slot")?;

    // Here, either the sender (this task) arrived first and needs to wait for a receiver,
    // or a receiver has already arrived and is waiting for a sender.
    let retval = {
      let mut exchange_state = sender_slot.0.lock();
      // Temporarily take ownership of the channel's waiting state so we can modify it;
      // the match statement below will advance the waiting state to the proper next state.
      if fault_id == 11 {
        // error!("At the point {}", fault_id);
        set_fault_item(0);
        *exchange_state = ExchangeState::ReceiverFinishedFirst;
      }
      let current_state = core::mem::replace(&mut *exchange_state, ExchangeState::Init);
      match current_state {
        ExchangeState::Init => {
          // Hold interrupts to avoid blocking & descheduling this task until we release the slot lock,
          // which is currently done automatically because the slot uses a MutexIrqSafe.
          *exchange_state = ExchangeState::WaitingForReceiver(WaitGuard::new(curr_task.clone()), msg);
          None
        }
        ExchangeState::WaitingForSender(receiver_to_notify) => {
          // The message has been sent successfully.
          *exchange_state = ExchangeState::SenderFinishedFirst(msg);
          // Notify the receiver task (outside of this match statement),
          // but DO NOT restore the sender slot to the channel yet;
          // that will be done once the receiver is also finished with the slot (in SenderFinishedFirst).
          Some(Ok(receiver_to_notify))
        }
        state => {
          error!("BUG: Sender (at beginning) in invalid state {:?}", state);
          *exchange_state = state;
          Some(Err("BUG: Sender (at beginning) in invalid state"))
        }
      }

      // here, the sender slot lock is dropped
    };
    // In the above block, we handled advancing the state of the exchange slot.
    // Now we need to handle other stuff (like notifying waiters) without holding the sender_slot lock.
    match retval {
      Some(Ok(receiver_to_notify)) => {
        drop(receiver_to_notify);
        return Ok(());
      }
      Some(Err(e)) => {
        // Restore the sender slot and notify waiting senders.
        self.channel.slot.replace_sender_slot(sender_slot);
        self.channel.waiting_senders.notify_one();
        return Err(e);
      }
      None => {
        // scheduler::schedule();
        microcall::thread_yield();
      }
    }

    // Here, the sender (this task) is waiting for a receiver
    loop {
      {
        let exchange_state = sender_slot.0.lock();
        match &*exchange_state {
          ExchangeState::WaitingForReceiver(blocked_sender, ..) => {
            if blocked_sender.task() != &curr_task {
              return Err("BUG: CURR TASK WAS DIFFERENT THAN BLOCKED SENDER");
            }
            blocked_sender.block_again();
          }
          _ => break,
        }
      }
      // scheduler::schedule();
      microcall::thread_yield();
    }

    // Here, we are at the rendezvous point
    let retval = {
      let mut exchange_state = sender_slot.0.lock();
      // Temporarily take ownership of the channel's waiting state so we can modify it;
      // the match statement below will advance the waiting state to the proper next state.
      let current_state = core::mem::replace(&mut *exchange_state, ExchangeState::Init);
      match current_state {
        ExchangeState::ReceiverFinishedFirst => {
          // Ready to transfer another message.
          *exchange_state = ExchangeState::Init;
          if fault_id == 13 {
            set_fault_item(0);
            *exchange_state = ExchangeState::ReceiverFinishedFirst;
          }
          Ok(())
        }
        state => {
          error!("BUG: Sender (while waiting) in invalid state {:?}", state);
          *exchange_state = state;
          Err("BUG: Sender (while waiting) in invalid state")
        }
      }

    };
    if retval.is_ok() {
      // Restore the receiver slot now that the receiver is finished, and notify waiting receivers.
      self.channel.slot.replace_receiver_slot(ReceiverSlot(sender_slot.0.clone()));
      self.channel.waiting_receivers.notify_one();
    }

    // Restore the sender slot and notify waiting senders.
    // trace!("sender done, restoring slot");
    self.channel.slot.replace_sender_slot(sender_slot);

    self.channel.waiting_senders.notify_one();
    // trace!("sender done, returning from send().");
    retval

    /*
    loop {
        let mut wait_entry = self.channel.waiter.lock();
        // temporarily take ownership of the channel's waiting state so we can modify it.
        let current_state = core::mem::replace(&mut *wait_entry, RendezvousState::Init);
        match current_state {
            RendezvousState::Init => {
                let _held_interrupts = irq_safety::hold_interrupts();
                *wait_entry = RendezvousState::Waiting(WaitGuard::new(curr_task.clone()), Some(msg));
                // interrupts are re-enabled here
            }
            RendezvousState::Waiting(task_to_notify, dest) => {
                *dest = Some(msg);
                let _held_interrupts = irq_safety::hold_interrupts();
                *task_to_notify = WaitGuard::new(curr_task.clone());
                drop(task_to_notify); // notifies the receiver
            }
        };
        let old_state = core::mem::replace(&mut wait_entry, new_state);
    }
    */
  }

  /// Tries to send the message, only succeeding if a receiver is ready and waiting.
  ///
  /// If a receiver was not ready, it returns the `msg` back to the caller without blocking.
  ///
  /// Note that if the non-blocking `try_send` and `try_receive` functions are only ever used,
  /// then the message will never be delivered because the sender and receiver cannot possibly rendezvous.
  pub fn try_send(&self, _msg: T) -> Result<(), T> {
    unimplemented!()
  }
}

/// The receiver side of a channel.
#[derive(Clone)]
pub struct Receiver<T: Send> {
  channel: Arc<Channel<T>>,
}
impl <T: Send> Receiver<T> {
  /// Receive a message, blocking until a sender is ready.
  ///
  /// Returns the message if it was received properly,
  /// otherwise returns an error.
  pub fn receive(&self) -> Result<T, &'static str> {

    let fault_id = get_fault_item();
    if fault_id == 7 {
      set_fault_item(0);
      unsafe { *(0x5050DEADBEEF as *mut usize) = 0x5555_5555_5555; }
    }

    if fault_id == 4 {
      set_fault_item(0);
      unsafe {
        let self_ptr = &(self) as *const _ as usize;
        let p1 = (self_ptr) as *mut usize;
        ptr::write(p1,0xDEADBEEF);
      }
    }

    // trace!("rendezvous: receive() entry");
    let curr_task = TaskRef(microcall::get_tid());

    // obtain a receiver-side exchange slot, blocking if necessary
    let receiver_slot = self.channel.take_receiver_slot().map_err(|_| "failed to take_receiver_slot")?;

    // Here, either the receiver (this task) arrived first and needs to wait for a sender,
    // or a sender has already arrived and is waiting for a receiver.
    let retval = {
      let mut exchange_state = receiver_slot.0.lock();

      if fault_id == 6 {
        set_fault_item(0);
        let _ = core::mem::replace(&mut *exchange_state, ExchangeState::ReceiverFinishedFirst);
      }

      // Temporarily take ownership of the channel's waiting state so we can modify it;
      // the match statement below will advance the waiting state to the proper next state.
      let current_state = core::mem::replace(&mut *exchange_state, ExchangeState::Init);
      match current_state {
        ExchangeState::Init => {
          // Hold interrupts to avoid blocking & descheduling this task until we release the slot lock,
          // which is currently done automatically because the slot uses a MutexIrqSafe.
          *exchange_state = ExchangeState::WaitingForSender(WaitGuard::new(curr_task.clone()));
          None
        }
        ExchangeState::WaitingForReceiver(sender_to_notify, msg) => {
          // The message has been received successfully!
          *exchange_state = ExchangeState::ReceiverFinishedFirst;
          // Notify the sender task (outside of this match statement),
          // but DO NOT restore the receiver slot to the channel yet;
          // that will be done once the sender is also finished with the slot (in ReceiverFinishedFirst).
          Some(Ok((sender_to_notify, msg)))
        }
        state => {
          error!("BUG: Receiver (at beginning) in invalid state {:?}", state);
          *exchange_state = state;
          Some(Err("BUG: Receiver (at beginning) in invalid state"))
        }
      }

      // here, the receiver slot lock is dropped
    };
    // In the above block, we handled advancing the state of the exchange slot.
    // Now we need to handle other stuff (like notifying waiters) without holding the receiver_slot lock.
    match retval {
      Some(Ok((sender_to_notify, msg))) => {
        if fault_id == 8 {
          set_fault_item(0);
          let self_ptr = &(msg) as *const _ as usize;
          let p1 = (self_ptr) as *mut usize;
          unsafe{ptr::write(p1,0x0)};
        }
        drop(sender_to_notify);
        return Ok(msg);
      }
      Some(Err(e)) => {
        // Restore the receiver slot and notify waiting receivers.
        self.channel.slot.replace_receiver_slot(receiver_slot);
        self.channel.waiting_receivers.notify_one();
        return Err(e);
      }
      None => {
        // scheduler::schedule();
        microcall::thread_yield();
      }
    }

    // Here, the receiver (this task) is waiting for a sender
    loop {
      {
        let exchange_state = receiver_slot.0.lock();
        match &*exchange_state {
          ExchangeState::WaitingForSender(blocked_receiver) => {
            warn!("spurious wakeup while receiver is WaitingForSender... re-blocking task.");
            if blocked_receiver.task() != &curr_task {
              return Err("BUG: CURR TASK WAS DIFFERENT THAN BLOCKED RECEIVER");
            }
            blocked_receiver.block_again();
          }
          _ => break,
        }
      }
      // scheduler::schedule();
      microcall::thread_yield();
    }


    // Here, we are at the rendezvous point
    let retval = {
      let mut exchange_state = receiver_slot.0.lock();
      // Temporarily take ownership of the channel's waiting state so we can modify it;
      // the match statement below will advance the waiting state to the proper next state.
      let current_state = core::mem::replace(&mut *exchange_state, ExchangeState::Init);
      match current_state {
        ExchangeState::SenderFinishedFirst(msg) => {
          // Ready to transfer another message.
          *exchange_state = ExchangeState::Init;
          Ok(msg)
        }
        state => {
          error!("BUG: Receiver (at end) in invalid state {:?}", state);
          *exchange_state = state;
          Err("BUG: Receiver (at end) in invalid state")
        }
      }

    };
    if retval.is_ok() {
      // Restore the sender slot now that the sender is finished, and notify waiting senders.
      self.channel.slot.replace_sender_slot(SenderSlot(receiver_slot.0.clone()));
      self.channel.waiting_senders.notify_one();
    }

    // Restore the receiver slot and notify waiting receivers.
    // trace!("receiver done, restoring slot");
    self.channel.slot.replace_receiver_slot(receiver_slot);
    self.channel.waiting_receivers.notify_one();
    // trace!("rendezvous: receiver done, returning from receive().");
    retval
  }

  /// Tries to receive a message, only succeeding if a sender is ready and waiting.
  ///
  /// If the sender was not ready, it returns an error without blocking.
  ///
  /// Note that if the non-blocking `try_send` and `try_receive` functions are only ever used,
  /// then the message will never be delivered because the sender and receiver cannot possibly rendezvous.
  pub fn try_receive(&self) -> Result<T, &'static str> {
    unimplemented!()
  }
}


// TODO: implement drop for sender and receiver in order to notify the other side of a disconnect
