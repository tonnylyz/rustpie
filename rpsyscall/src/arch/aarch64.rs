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
                    "svc #0",
                    in("x8") $a,
                    $(in("x0") $b,
                    $(in("x1") $c,
                    $(in("x2") $d,
                    $(in("x3") $e,
                    $(in("x4") $f,)?)?)?)?)?

                    $(lateout("x0") $oa,
                    $(lateout("x1") $ob,
                    $(lateout("x2") $oc,
                    $(lateout("x3") $od,
                    $(lateout("x4") $oe,)?)?)?)?)?
                    lateout("x7") ret,
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