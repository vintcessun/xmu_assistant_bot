#[macro_export]
macro_rules! register_handler_with_help {
    (
        $(command = [ $($cmd_handlers:path),* $(,)? ])?
        $(,)?
        $(other = [ $($other_handlers:path),* $(,)? ])?
    ) => {
        use $crate::abi::{logic_import::*, message::from_str};

        // 1. 转发给注册宏，包含所有 handler 和生成的 HelpHandler
        register_handlers!(
            $($($cmd_handlers,)*)?
            $($($other_handlers,)*)?
            HelpHandler
        );


        // 2. 生成真正的 help 处理函数
        #[handler(
            msg_type = Message,
            command = "help",
            echo_cmd = true,
            help_msg = "用法:/help\n功能:显示帮助信息"
        )]
        pub async fn help(ctx: Context) -> Result<()> {
            // 直接使用 concatcp! 拼接所有常量字符串
            // 在每一项后面手动加上换行符 "\n" 以保证格式
            const ALL_HELP: &'static str = const_format::concatcp!(
                HelpHandler::HELP_MSG, "\n",
                $( $( <$cmd_handlers as BuildHelp>::HELP_MSG, "\n", )* )?
            );

            ctx.send_message_async(from_str(ALL_HELP));
            Ok(())
        }
    };
}
