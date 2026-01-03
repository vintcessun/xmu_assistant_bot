#[macro_export]
macro_rules! box_new {
    // 核心修复：支持结构体字段中嵌套另一个 box_new! 或其他表达式
    ($t:ty, { $($field:ident : $val:expr),* $(,)? }) => {{
        // 在安全上下文中计算所有值，这里允许 $val 是另一个宏展开
        $( let $field = $val; )*

        let mut b = Box::<$t>::new_uninit();
        let ptr = b.as_mut_ptr();
        unsafe {
            $(
                std::ptr::addr_of_mut!((*ptr).$field).write($field);
            )*
            b.assume_init()
        }
    }};

    // 修复枚举变体嵌套：直接将整个变体构造视为一个表达式
    ($t:ty, $val:expr) => {{
        let val = $val;
        let mut b = Box::<$t>::new_uninit();
        unsafe {
            b.as_mut_ptr().write(val);
            b.assume_init()
        }
    }};
}
