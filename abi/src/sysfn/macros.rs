macro_rules! def_sysfn {
    (
        $(
            @$sysno:ident
            $name:ident(
                $($arg:ident: $typ:ty,)*
            ) -> $ret:tt // NOTE: $ret:ty causes error when $ret is ! (never)
        )+
    ) => {
        $(
            def_sysfn!($sysno $name($($arg: $typ,)*) -> $ret);
        )+
    };

    ($sysno:ident $name:ident() -> !) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name() -> ! {
            def_sysfn!(@__call_noret $sysno! 0, 0, 0, 0, 0, 0, 0);
        }
    };

    ($sysno:ident $name:ident() -> $ret:ty) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name() -> $ret {
            def_sysfn!(@__call $sysno! 0, 0, 0, 0, 0, 0, 0);
        }
    };

    ($sysno:ident $name:ident(
        $arg0:ident: $ty0:ty,
    ) -> !) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name(
            $arg0: $ty0,
        ) -> ! {
            def_sysfn!(@__call_noret $sysno! $arg0, 0, 0, 0, 0, 0, 0);
        }
    };

    ($sysno:ident $name:ident(
        $arg0:ident: $ty0:ty,
    ) -> $ret:ty) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name(
            $arg0: $ty0,
        ) -> $ret {
            def_sysfn!(@__call $sysno! $arg0, 0, 0, 0, 0, 0, 0);
        }
    };

    ($sysno:ident $name:ident(
        $arg0:ident: $ty0:ty,
        $arg1:ident: $ty1:ty,
    ) -> !) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name(
            $arg0: $ty0,
            $arg1: $ty1,
        ) -> ! {
            def_sysfn!(@__call_noret $sysno! $arg0, $arg1, 0, 0, 0, 0, 0);
        }
    };

    ($sysno:ident $name:ident(
        $arg0:ident: $ty0:ty,
        $arg1:ident: $ty1:ty,
    ) -> $ret:ty) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name(
            $arg0: $ty0,
            $arg1: $ty1,
        ) -> $ret {
            def_sysfn!(@__call $sysno! $arg0, $arg1, 0, 0, 0, 0, 0);
        }
    };

    ($sysno:ident $name:ident(
        $arg0:ident: $ty0:ty,
        $arg1:ident: $ty1:ty,
        $arg2:ident: $ty2:ty,
    ) -> !) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name(
            $arg0: $ty0,
            $arg1: $ty1,
            $arg2: $ty2,
        ) -> ! {
            def_sysfn!(@__call_noret $sysno! $arg0, $arg1, $arg2, 0, 0, 0, 0);
        }
    };

    ($sysno:ident $name:ident(
        $arg0:ident: $ty0:ty,
        $arg1:ident: $ty1:ty,
        $arg2:ident: $ty2:ty,
    ) -> $ret:ty) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name(
            $arg0: $ty0,
            $arg1: $ty1,
            $arg2: $ty2,
        ) -> $ret {
            def_sysfn!(@__call $sysno! $arg0, $arg1, $arg2, 0, 0, 0, 0);
        }
    };

    ($sysno:ident $name:ident(
        $arg0:ident: $ty0:ty,
        $arg1:ident: $ty1:ty,
        $arg2:ident: $ty2:ty,
        $arg3:ident: $ty3:ty,
    ) -> !) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name(
            $arg0: $ty0,
            $arg1: $ty1,
            $arg2: $ty2,
            $arg3: $ty3,
        ) -> ! {
            def_sysfn!(@__call_noret $sysno! $arg0, $arg1, $arg2, $arg3, 0, 0, 0);
        }
    };

    ($sysno:ident $name:ident(
        $arg0:ident: $ty0:ty,
        $arg1:ident: $ty1:ty,
        $arg2:ident: $ty2:ty,
        $arg3:ident: $ty3:ty,
    ) -> $ret:ty) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name(
            $arg0: $ty0,
            $arg1: $ty1,
            $arg2: $ty2,
            $arg3: $ty3,
        ) -> $ret {
            def_sysfn!(@__call $sysno! $arg0, $arg1, $arg2, $arg3, 0, 0, 0);
        }
    };

    ($sysno:ident $name:ident(
        $arg0:ident: $ty0:ty,
        $arg1:ident: $ty1:ty,
        $arg2:ident: $ty2:ty,
        $arg3:ident: $ty3:ty,
        $arg4:ident: $ty4:ty,
    ) -> !) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name(
            $arg0: $ty0,
            $arg1: $ty1,
            $arg2: $ty2,
            $arg3: $ty3,
            $arg4: $ty4,
        ) -> ! {
            def_sysfn!(@__call_noret $sysno! $arg0, $arg1, $arg2, $arg3, $arg4, 0, 0);
        }
    };

    ($sysno:ident $name:ident(
        $arg0:ident: $ty0:ty,
        $arg1:ident: $ty1:ty,
        $arg2:ident: $ty2:ty,
        $arg3:ident: $ty3:ty,
        $arg4:ident: $ty4:ty,
    ) -> $ret:ty) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name(
            $arg0: $ty0,
            $arg1: $ty1,
            $arg2: $ty2,
            $arg3: $ty3,
            $arg4: $ty4,
        ) -> $ret {
            def_sysfn!(@__call $sysno! $arg0, $arg1, $arg2, $arg3, $arg4, 0, 0);
        }
    };

    ($sysno:ident $name:ident(
        $arg0:ident: $ty0:ty,
        $arg1:ident: $ty1:ty,
        $arg2:ident: $ty2:ty,
        $arg3:ident: $ty3:ty,
        $arg4:ident: $ty4:ty,
        $arg5:ident: $ty5:ty,
    ) -> !) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name(
            $arg0: $ty0,
            $arg1: $ty1,
            $arg2: $ty2,
            $arg3: $ty3,
            $arg4: $ty4,
            $arg5: $ty5,
        ) -> ! {
            def_sysfn!(@__call_noret $sysno! $arg0, $arg1, $arg2, $arg3, $arg4, $arg5, 0);
        }
    };

    ($sysno:ident $name:ident(
        $arg0:ident: $ty0:ty,
        $arg1:ident: $ty1:ty,
        $arg2:ident: $ty2:ty,
        $arg3:ident: $ty3:ty,
        $arg4:ident: $ty4:ty,
        $arg5:ident: $ty5:ty,
    ) -> $ret:ty) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name(
            $arg0: $ty0,
            $arg1: $ty1,
            $arg2: $ty2,
            $arg3: $ty3,
            $arg4: $ty4,
            $arg5: $ty5,
        ) -> $ret {
            def_sysfn!(@__call $sysno! $arg0, $arg1, $arg2, $arg3, $arg4, $arg5, 0);
        }
    };

    ($sysno:ident $name:ident(
        $arg0:ident: $ty0:ty,
        $arg1:ident: $ty1:ty,
        $arg2:ident: $ty2:ty,
        $arg3:ident: $ty3:ty,
        $arg4:ident: $ty4:ty,
        $arg5:ident: $ty5:ty,
        $arg6:ident: $ty6:ty,
    ) -> !) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name(
            $arg0: $ty0,
            $arg1: $ty1,
            $arg2: $ty2,
            $arg3: $ty3,
            $arg4: $ty4,
            $arg5: $ty5,
            $arg6: $ty6,
        ) -> ! {
            def_sysfn!(@__call_noret $sysno! arg0, arg1, arg2, arg3, arg4, arg5, arg6);
        }
    };

    ($sysno:ident $name:ident(
        $arg0:ident: $ty0:ty,
        $arg1:ident: $ty1:ty,
        $arg2:ident: $ty2:ty,
        $arg3:ident: $ty3:ty,
        $arg4:ident: $ty4:ty,
        $arg5:ident: $ty5:ty,
        $arg6:ident: $ty6:ty,
    ) -> $ret:ty) => {
        #[inline(always)]
        #[no_mangle]
        pub extern "C" fn $name(
            $arg0: $ty0,
            $arg1: $ty1,
            $arg2: $ty2,
            $arg3: $ty3,
            $arg4: $ty4,
            $arg5: $ty5,
            $arg6: $ty6,
        ) -> $ret {
            def_sysfn!(@__call $sysno! arg0, arg1, arg2, arg3, arg4, arg5, arg6);
        }
    };

    (@__call $sysno:ident!
        $arg0:expr,
        $arg1:expr,
        $arg2:expr,
        $arg3:expr,
        $arg4:expr,
        $arg5:expr,
        $arg6:expr
    ) => {
        let ret: usize;

        cfg_if::cfg_if! {
            if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
                let arg0: usize = $arg0 as _;
                let arg1: usize = $arg1 as _;
                let arg2: usize = $arg2 as _;
                let arg3: usize = $arg3 as _;
                let arg4: usize = $arg4 as _;
                let arg5: usize = $arg5 as _;
                let arg6: usize = $arg6 as _;
                unsafe {
                    core::arch::asm!(
                        "ecall",
                        inlateout("a0") arg0 => ret,
                        in("a1") arg1,
                        in("a2") arg2,
                        in("a3") arg3,
                        in("a4") arg4,
                        in("a5") arg5,
                        in("a6") arg6,
                        in("a7") $sysno,
                    );
                }
            } else {
                compile_error!("unsupported architecture");
            }
        }

        return ret.into();
    };

    (@__call_noret $sysno:ident!
        $arg0:expr,
        $arg1:expr,
        $arg2:expr,
        $arg3:expr,
        $arg4:expr,
        $arg5:expr,
        $arg6:expr
    ) => {
        cfg_if::cfg_if! {
            if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
                let arg0: usize = $arg0 as _;
                let arg1: usize = $arg1 as _;
                let arg2: usize = $arg2 as _;
                let arg3: usize = $arg3 as _;
                let arg4: usize = $arg4 as _;
                let arg5: usize = $arg5 as _;
                let arg6: usize = $arg6 as _;
                unsafe {
                    core::arch::asm!(
                        "ecall",
                        in("a0") arg0,
                        in("a1") arg1,
                        in("a2") arg2,
                        in("a3") arg3,
                        in("a4") arg4,
                        in("a5") arg5,
                        in("a6") arg6,
                        in("a7") $sysno,
                    );
                }
            } else {
                compile_error!("unsupported architecture");
            }
        }
        unreachable!();
    }
}
pub(crate) use def_sysfn;
