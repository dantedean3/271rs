#[macro_export]
macro_rules! choice {
    ($e:expr, $f:expr, $g:expr) => {{
        let _e = $e;
        let _f = $f;
        let _g = $g;
        (_e & _f) ^ ((!_e) & _g)
    }};
}

#[macro_export]
macro_rules! median {
    ($e:expr, $f:expr, $g:expr) => {{
        let _e = $e;
        let _f = $f;
        let _g = $g;
        (_e & _f) | (_e & _g) | (_f & _g)
    }};
}

#[macro_export]
macro_rules! rotate {
    ($x:expr, $n:expr) => {{
        let _x = $x;
        let _bits: u32 = (core::mem::size_of_val(&_x) * 8) as u32;
        let _n: u32 = {
            let raw = ($n) as u32;
            if _bits == 0 { 0 } else { raw % _bits }
        };
        if _n == 0 {
            _x
        } else {
            let _right = _x.checked_shr(_n).unwrap_or(0);
            let _left = _x.checked_shl(_bits - _n).unwrap_or(0);
            _right | _left
        }
    }};
}
