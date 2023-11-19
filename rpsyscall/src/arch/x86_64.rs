pub type Error = usize;

macro_rules! syscall {

    ($($name:ident($a:ident, $($b:ident, $($c:ident, $($d:ident, $($e:ident, $($f:ident, )?)?)?)?)?) -> ($($oa:ident:$ta:tt, $($ob:ident:$tb:tt, $($oc:ident:$tc:tt, $($od:ident:$td:tt, $($oe:ident:$te:tt, )?)?)?)?)?);)+) => {
        $(
            #[inline(always)]
            #[allow(unused_parens)]
            #[allow(dead_code)]
            pub fn $name($a: usize, $($b: usize, $($c: usize, $($d: usize, $($e: usize, $($f: usize)?)?)?)?)?) -> Result<($($ta$(, $tb$(, $tc$(, $td$(, $te)?)?)?)?)?), Error> {
                let ret: usize;
                $(let $oa: $ta;
                $(let $ob: $tb;
                $(let $oc: $tc;
                $(let $od: $td;
                $(let $oe: $te;)?)?)?)?)?
                unsafe {
                core::arch::asm!(
                    "syscall",
                    in("rax") $a,
                    $(in("rdi") $b,
                    $(in("rsi") $c,
                    $(in("rdx") $d,
                    $(in("r10") $e,
                    $(in("r8") $f,)?)?)?)?)?

                    $(lateout("rdi") $oa,
                    $(lateout("rsi") $ob,
                    $(lateout("rdx") $oc,
                    $(lateout("r10") $od,
                    $(lateout("r8") $oe,)?)?)?)?)?
                    lateout("rax") ret,
                    out("rcx") _, // rip
                    out("r11") _, // rflags
                    options(nostack),
                );
                }
                if (ret == 0) {
                    Ok(($($oa$(, $ob$(, $oc$(, $od$(, $oe)?)?)?)?)?))
                } else {
                    Err(ret)
                }
            }
        )+
    };

}

syscall! {
    syscall_0_0(a, ) -> ();
    syscall_1_0(a, b, ) -> ();
    syscall_2_0(a, b, c, ) -> ();
    syscall_3_0(a, b, c, d, ) -> ();
    syscall_4_0(a, b, c, d, e, ) -> ();
    syscall_5_0(a, b, c, d, e, f, ) -> ();
    syscall_0_1(a, ) -> (oa: usize, );
    syscall_1_1(a, b, ) -> (oa: usize, );
    syscall_2_1(a, b, c, ) -> (oa: usize, );
    syscall_0_2(a, ) -> (oa: usize, ob: usize, );
    syscall_4_1(a, b, c, d, e, ) -> (oa: usize, );
    syscall_0_5(a, ) -> (oa: usize, ob: usize, oc: usize, od: usize, oe: usize, );
    syscall_5_5(a, b, c, d, e, f, ) -> (oa: usize, ob: usize, oc: usize, od: usize, oe: usize, );
}