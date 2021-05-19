use alloc::borrow::Cow;
use alloc::string::String;

use addr2line::Context;

use core::u32;

pub fn resolve(
    ctxt: Option<&Context>,
    offset: u64,
    addr: *mut u8,
    cb: &mut dyn FnMut(&super::Symbol),
) -> Result<(), addr2line::gimli::read::Error> {
    let addr = (addr as u64 - offset) as usize;

    // Try to resolve an address within a context:
    let (file, line, fn_name): (
        Option<String>,
        Option<u64>,
        Option<
            addr2line::FunctionName<
                addr2line::gimli::EndianReader<
                    addr2line::gimli::RunTimeEndian,
                    alloc::rc::Rc<[u8]>,
                >,
            >,
        >,
    ) = ctxt.map_or_else(
        || (None, None, None),
        |ctxt| {
            let frame_iter = ctxt.find_frames(addr as u64);
            if frame_iter.is_ok() {
                let mut frame_iter = frame_iter.unwrap();
                let frame_result = frame_iter.next();
                if frame_result.is_ok() {
                    let maybe_first_frame = frame_result.unwrap();
                    if maybe_first_frame.is_some() {
                        let first_frame = maybe_first_frame.unwrap();
                        let fn_name = first_frame.function;
                        let location = ctxt.find_location(addr as u64);

                        return match location {
                            Ok(Some(l)) => (l.file, l.line, fn_name),
                            _ => (None, None, fn_name),
                        };
                    }
                }
            }
            (None, None, None)
        },
    );

    let sym = super::Symbol {
        inner: Symbol::new(addr, file, line, fn_name),
    };

    Ok(cb(&sym))
}

pub struct Symbol {
    addr: usize,
    file: Option<String>,
    line: Option<u64>,
    fn_name: Option<
        addr2line::FunctionName<
            addr2line::gimli::EndianReader<addr2line::gimli::RunTimeEndian, alloc::rc::Rc<[u8]>>,
        >,
    >,
}

impl Symbol {
    fn new(
        addr: usize,
        file: Option<String>,
        line: Option<u64>,
        fn_name: Option<
            addr2line::FunctionName<
                addr2line::gimli::EndianReader<
                    addr2line::gimli::RunTimeEndian,
                    alloc::rc::Rc<[u8]>,
                >,
            >,
        >,
    ) -> Symbol {
        Symbol {
            addr,
            file,
            line,
            fn_name,
        }
    }

    pub fn name(&self) -> Option<Cow<str>> {
        self.fn_name.as_ref().map(|f| f.demangle().unwrap())
    }

    pub fn addr(&self) -> Option<*mut u8> {
        Some(self.addr as *mut u8)
    }

    pub fn filename(&self) -> Option<&str> {
        self.file.as_ref().map(|f| f.as_ref())
    }

    pub fn lineno(&self) -> Option<u32> {
        self.line.and_then(|l| {
            if l > (u32::MAX as u64) {
                None
            } else {
                Some(l as u32)
            }
        })
    }
}
