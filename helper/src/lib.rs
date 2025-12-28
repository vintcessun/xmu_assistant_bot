extern crate proc_macro;
use heck::AsPascalCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Error, Ident, ItemFn, ItemStruct, LitStr, Result, Type,
    parse::{Parse, ParseStream},
    parse_macro_input, token,
};

// 1. 定义一个结构体来解析属性参数: #[api("/path", ResponseType)]
struct ApiAttr {
    path: LitStr,
    _comma: token::Comma,
    response_type: Type,
}

impl Parse for ApiAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ApiAttr {
            path: input.parse()?,
            _comma: input.parse()?,
            response_type: input.parse()?,
        })
    }
}

#[proc_macro_attribute]
pub fn api(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 2. 解析属性参数
    let args = parse_macro_input!(attr as ApiAttr);
    let path = args.path.value();
    let response_type = args.response_type;

    // 校验斜杠
    if !path.starts_with('/') || path.len() < 2 {
        return Error::new(
            args.path.span(),
            "Must start with '/' and have an action name (e.g., #[api(\"/send_msg\", MyResponse)])",
        )
        .to_compile_error()
        .into();
    }

    let action_str = &path[1..];

    // 3. 解析结构体本身
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    // 4. 生成代码，包含关联类型
    let expanded = quote! {
        #input

        impl Params for #name {
            type Response = #response_type; // 自动绑定关联类型

            fn get_action(&self) -> &'static str {
                #action_str
            }
        }
    };

    expanded.into()
}

#[proc_macro]
pub fn define_default_type(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let parts: Vec<&str> = input_str.split(',').map(|s| s.trim()).collect();

    if parts.len() != 3 {
        return quote! { compile_error!("Expected 3 arguments: name, type, default_val"); }.into();
    }

    let name = syn::parse_str::<syn::Ident>(parts[0]).unwrap();
    let ty = syn::parse_str::<syn::Type>(parts[1]).unwrap();
    let default_val = syn::parse_str::<syn::Expr>(parts[2]).unwrap();

    let expanded = quote! {
        #[derive(Debug, Clone, PartialEq, serde::Serialize)]
        pub struct #name(pub #ty);

        impl Default for #name {
            fn default() -> Self {
                Self(#default_val)
            }
        }

        impl<'de> serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de> {
                let opt = Option::<#ty>::deserialize(deserializer)?;
                Ok(Self(opt.unwrap_or_else(|| #default_val)))
            }
        }

        impl std::ops::Deref for #name {
            type Target = #ty;
            fn deref(&self) -> &Self::Target { &self.0 }
        }
    };
    expanded.into()
}

#[proc_macro_attribute]
pub fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let vis = &input_fn.vis;
    let body = &input_fn.block;
    let target_type_ident = parse_macro_input!(attr as Ident);

    // 关键点 1: 确保生成的结构体标识符拥有原始函数名的 Span 信息
    // 这会让 IDE 认为这个结构体就是在这里定义的
    let struct_name = format_ident!(
        "{}Handler",
        AsPascalCase(fn_name.to_string()).to_string(),
        span = fn_name.span()
    );

    let hidden_impl = format_ident!("__hidden_{}_impl", fn_name);

    let expanded = quote! {
        // 关键点 2: 保留原始函数的“足迹”，但把它变成一个对 IDE 友好的常量或类型别名
        // 这样当你输入函数名时，RA 会提示你它被映射到了对应的 Handler 类
        #[allow(non_upper_case_globals)]
        #vis const #fn_name: #struct_name = #struct_name;

        // 1. 定义隐藏的实现函数
        async fn #hidden_impl<T>(mut ctx: Context<T, #target_type_ident>) -> anyhow::Result<()>
        where T: BotClient + BotHandler + std::fmt::Debug + 'static
        {
            let plugin_logic = |mut ctx: Context<T, #target_type_ident>| async move {
                let result: anyhow::Result<()> = { #body };
                result
            };
            plugin_logic(ctx).await
        }

        // 2. 生成 Handler 结构体
        #[derive(Clone, Default, Debug)]
        #vis struct #struct_name;

        #[async_trait::async_trait]
        impl<T, M> Handler<T, M> for #struct_name
        where
            T: BotClient + BotHandler + std::fmt::Debug + 'static,
            M: MessageType + std::fmt::Debug + Send + Sync + 'static,
        {
            async fn handle(&self, ctx: Context<T, M>) -> anyhow::Result<()> {
                if ctx.message.get_type() == Type::#target_type_ident {
                    let typed_ctx = unsafe {
                        std::mem::transmute::<Context<T, M>, Context<T, #target_type_ident>>(ctx)
                    };
                    #hidden_impl(typed_ctx).await
                } else {
                    Ok(())
                }
            }
        }
    };

    TokenStream::from(expanded)
}
