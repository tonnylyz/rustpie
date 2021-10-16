pub type Error = usize;

macro_rules! syscall {

    ($($name:ident($a:ident, $($b:ident, $($c:ident, $($d:ident, $($e:ident, $($f:ident, )?)?)?)?)?) -> ($($oa:ident:$ta:tt, $($ob:ident:$tb:tt, $($oc:ident:$tc:tt, $($od:ident:$td:tt, $($oe:ident:$te:tt, )?)?)?)?)?);)+) => {
        $(
            #[inline(always)]
            pub fn $name($a: usize, $($b: usize, $($c: usize, $($d: usize, $($e: usize, $($f: usize)?)?)?)?)?) -> Result<($($ta$(, $tb$(, $tc$(, $td$(, $te)?)?)?)?)?), Error> {
                let ret: usize;
                $(let $oa: $ta;
                $(let $ob: $tb;
                $(let $oc: $tc;
                $(let $od: $td;
                $(let $oe: $te;)?)?)?)?)?
                unsafe {
                asm!(
                    "ecall #0",
                    in("x17") $a,
                    $(in("x10") $b,
                    $(in("x11") $c,
                    $(in("x12") $d,
                    $(in("x13") $e,
                    $(in("x14") $f,)?)?)?)?)?

                    $(lateout("x10") $oa,
                    $(lateout("x11") $ob,
                    $(lateout("x12") $oc,
                    $(lateout("x13") $od,
                    $(lateout("x14") $oe,)?)?)?)?)?
                    lateout("x16") ret,
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

syscall!{
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