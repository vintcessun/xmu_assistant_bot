extern crate proc_macro;
use darling::FromMeta;
use heck::AsPascalCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    Data, DeriveInput, Error, Expr, FnArg, GenericArgument, Ident, ItemFn, ItemStruct, LitBool,
    LitStr, Meta, Pat, Path, PathArguments, Result, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token,
};

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
    let args = parse_macro_input!(attr as ApiAttr);
    let path = args.path.value();
    let response_type = args.response_type;

    if !path.starts_with('/') || path.len() < 2 {
        return Error::new(
            args.path.span(),
            "Must start with '/' and have an action name (e.g., #[api(\"/send_msg\", MyResponse)])",
        )
        .to_compile_error()
        .into();
    }

    let action_str = &path[1..];

    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl Params for #name {
            type Response = #response_type;

            const ACTION: &'static str = #action_str;
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

#[derive(Debug, FromMeta)]
struct HandlerArgs {
    #[darling(default)]
    msg_type: Option<Ident>,
    #[darling(default)]
    command: Option<LitStr>,
    #[darling(default)]
    echo_cmd: bool,
}

impl Parse for HandlerArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut msg_type = None;
        let mut command = None;
        let mut echo_cmd = false;

        if input.is_empty() {
            return Ok(HandlerArgs {
                msg_type,
                command,
                echo_cmd,
            });
        }

        let pairs = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        for meta in pairs {
            let path = meta.path();
            if path.is_ident("msg_type") {
                if let Meta::NameValue(nv) = meta {
                    let expr = nv.value;
                    msg_type = Some(syn::parse2::<Ident>(quote!(#expr))?);
                }
            } else if path.is_ident("command") {
                if let Meta::NameValue(nv) = meta {
                    let expr = nv.value;
                    command = Some(syn::parse2::<LitStr>(quote!(#expr))?);
                }
            } else if path.is_ident("echo_cmd") {
                if let Meta::NameValue(nv) = meta {
                    let expr = nv.value;
                    // 解析 echo_cmd = true/false
                    let lit: LitBool = syn::parse2(quote!(#expr))?;
                    echo_cmd = lit.value;
                }
            } else {
                return Err(syn::Error::new_spanned(
                    path,
                    "Unknown attribute key, expected 'msg_type', 'command', or 'echo_cmd'",
                ));
            }
        }

        if echo_cmd {
            if !msg_type
                .as_ref()
                .map(|t| t.to_string() == "Message")
                .unwrap_or(false)
            {
                return Err(syn::Error::new_spanned(
                    &msg_type,
                    "When 'echo_cmd' is true, 'msg_type' must be 'Message'",
                ));
            }
        }

        Ok(HandlerArgs {
            msg_type,
            command,
            echo_cmd,
        })
    }
}

#[proc_macro_attribute]
pub fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let args = parse_macro_input!(attr as HandlerArgs);

    let fn_name = &input_fn.sig.ident;
    let vis = &input_fn.vis;
    let body = &input_fn.block;

    let target_type_ident = args.msg_type.clone().unwrap_or_else(|| format_ident!("M"));

    let struct_name = format_ident!(
        "{}Handler",
        AsPascalCase(fn_name.to_string()).to_string(),
        span = fn_name.span()
    );

    let hidden_impl = format_ident!("__hidden_{}_impl", fn_name);

    let type_filter = if let Some(ref ty) = args.msg_type {
        quote! { ctx.message.get_type() == Type::#ty }
    } else {
        quote! { true }
    };

    let command_filter = if let Some(ref cmd) = args.command {
        quote! {
            {
                ctx.get_message_text().starts_with(const_format::formatcp!("{}{}", config::get_command_prefix(), #cmd))
            }
        }
    } else {
        quote! { true }
    };

    let echo_logic = if args.echo_cmd {
        quote! {
            {
                let mut ctx = typed_ctx;
                ctx.set_echo();
                ctx
            }
        }
    } else {
        quote! {
            typed_ctx
        }
    };

    let (generics, target_type) = if args.msg_type.is_some() {
        (quote! { <T> }, quote! { #target_type_ident })
    } else {
        (
            quote! { <T, M: MessageType + std::fmt::Debug> },
            quote! { M },
        )
    };

    let expanded = quote! {
        #[allow(non_upper_case_globals)]
        #vis const #fn_name: #struct_name = #struct_name;

        async fn #hidden_impl #generics(mut ctx: Context<T, #target_type>) -> anyhow::Result<()>
        where T: BotClient + BotHandler + std::fmt::Debug + 'static
        {
            let result: anyhow::Result<()> = (async { #body }).await;;
            if let Err(e) = result{
                handle_error(ctx, stringify!(#fn_name), e).await;
            }
            Ok(())
        }

        #[derive(Clone, Default, Debug)]
        #vis struct #struct_name;

        #[async_trait::async_trait]
        impl<T, M> Handler<T, M> for #struct_name
        where
            T: BotClient + BotHandler + std::fmt::Debug + 'static,
            M: MessageType + std::fmt::Debug + Send + Sync + 'static,
        {
            async fn handle(&self, ctx: Context<T, M>) -> anyhow::Result<()> {
                if #type_filter && #command_filter {
                    let typed_ctx = unsafe {
                        std::mem::transmute::<Context<T, M>, Context<T, #target_type_ident>>(ctx)
                    };
                    let handle_ctx = #echo_logic;
                    #hidden_impl(handle_ctx).await
                } else {
                    Ok(())
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn register_handlers(input: TokenStream) -> TokenStream {
    let parser = Punctuated::<Path, Token![,]>::parse_terminated;
    let handlers = parse_macro_input!(input with parser);

    let spawns = handlers.iter().map(|handler| {
        quote! {
            {
                let ctx = context.clone();
                let handler_instance = #handler;
                tokio::spawn(async move {
                    if let Err(e) = handler_instance.handle(ctx).await {
                        tracing::error!("Handler [{}] 运行时错误: {:?}", stringify!(#handler), e);
                    }
                });
            }
        }
    });

    let expanded = quote! {
        #[allow(non_snake_case, dead_code)]
        pub fn dispatch_all_handlers<T, M>(context: Context<T, M>)
        where
            T: BotClient + BotHandler + std::fmt::Debug + Sync + Send + 'static,
            M: MessageType + std::fmt::Debug + Sync + Send + 'static,
        {
            #(#spawns)*
        }
    };

    TokenStream::from(expanded)
}

enum WrapState {
    Normal,
    Query,
}

struct JwApiArgs {
    url: String,
    app: String,
    field_name: String,
    wrapper_name: String,
    wrap_response: WrapState,
}

impl Parse for JwApiArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vars = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        let mut url: Option<String> = None;
        let mut app: Option<String> = None;
        let mut wrap_response = WrapState::Normal;
        let mut wrapper_name: Option<String> = None;

        for meta in vars {
            if let Meta::NameValue(nv) = meta {
                if nv.path.is_ident("url") {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = nv.value
                    {
                        url = Some(s.value());
                    }
                } else if nv.path.is_ident("app") {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = nv.value
                    {
                        app = Some(s.value());
                    }
                } else if nv.path.is_ident("wrapper_name") {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = nv.value
                    {
                        wrapper_name = Some(s.value());
                    }
                } else {
                    return Err(syn::Error::new_spanned(
                        nv.path,
                        "Unknown attribute key, expected 'url', 'app', 'wrap_response', or 'wrapper_name'",
                    ));
                }
            }
        }

        let url = url.ok_or_else(|| {
            syn::Error::new(
                input.span(),
                "Missing required attribute: `url` (e.g., #[jw_api(url = \"...\")])",
            )
        })?;

        let app = app.ok_or_else(|| {
            syn::Error::new(
                input.span(),
                "Missing required attribute: `app` (e.g., #[jw_api(app = \"...\")])",
            )
        })?;

        if !url.ends_with(".do") {
            return Err(syn::Error::new(
                input.span(),
                "The url is invalid because it does not end with .do",
            ));
        }

        let field_name = url
            .split('/')
            .last()
            .and_then(|s| s.split('.').next())
            .ok_or(syn::Error::new(
                input.span(),
                "The url may be invalid because it does not contain a valid api name",
            ))?
            .to_string();

        if field_name.find("Xs").is_some() {
            wrap_response = WrapState::Query;
        }

        let wrapper_name = match wrapper_name {
            Some(name) => name,
            None => match wrap_response {
                WrapState::Normal => "datas".to_string(),
                WrapState::Query => "data".to_string(),
            },
        };

        Ok(JwApiArgs {
            url,
            app,
            field_name,
            wrap_response,
            wrapper_name,
        })
    }
}

#[proc_macro_attribute]
pub fn jw_api(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as JwApiArgs);

    let input_struct = parse_macro_input!(input as ItemStruct);
    let original_ident = &input_struct.ident;

    let response_item_ident = format_ident!("{}Response", original_ident);
    let data_api_ident = format_ident!("{}DataApi", original_ident);
    let datas_ident = format_ident!("{}Datas", original_ident);

    let data_name = format_ident!("{}", args.wrapper_name);

    let vis = &input_struct.vis;
    let fields = &input_struct.fields;
    let url_val = args.url;
    let app_val = args.app;

    let field_name_from_url = args.field_name;

    let dynamic_field_ident = format_ident!("{}", field_name_from_url);

    let expanded = match args.wrap_response {
        WrapState::Normal => quote! {
            #[derive(Deserialize, Debug, Clone, Default)]
            #[serde(rename_all = "UPPERCASE")]
            #vis struct #response_item_ident
            #fields

            #[async_trait::async_trait]
            impl JwAPI for #original_ident {
                const URL_DATA: &'static str = #url_val;
                const APP_ENTRANCE: &'static str = #app_val;
            }

            #[derive(Deserialize, Debug, Clone, Default)]
            #[serde(rename_all = "camelCase")]
            #vis struct #data_api_ident {
                pub rows: Vec<#response_item_ident>,
                // pub ext_params: serde::de::IgnoredAny,
                // pub page_number: serde::de::IgnoredAny,
                // pub page_size: serde::de::IgnoredAny,
                // pub total_size: serde::de::IgnoredAny,
            }

            #[derive(Deserialize, Debug, Clone, Default)]
            #vis struct #datas_ident {
                pub #dynamic_field_ident: #data_api_ident,
            }

            #[derive(Deserialize, Debug, Clone, Default)]
            #vis struct #original_ident {
                pub code: String,
                pub #data_name: #datas_ident,
            }

            impl #original_ident {
                pub async fn call<D: Serialize + Sync>(castgc: &str, data: &D) -> Result<#original_ident> {
                    let client = crate::api::xmu_service::jw::get_castgc_client(castgc);
                    let res_auth = client.get(#original_ident::APP_ENTRANCE).await?;

                    let resp = client.post(#original_ident::URL_DATA, data).await?.json().await?;
                    Ok(resp)
                }
            }
        },
        WrapState::Query => quote! {
            #[derive(Deserialize, Debug, Clone, Default)]
            #[serde(rename_all = "UPPERCASE")]
            #vis struct #response_item_ident
            #fields

            #[async_trait::async_trait]
            impl JwAPI for #original_ident {
                const URL_DATA: &'static str = #url_val;
                const APP_ENTRANCE: &'static str = #app_val;
            }

            #[derive(Deserialize, Debug, Clone, Default)]
            #vis struct #original_ident {
                pub #data_name: Vec<#response_item_ident>,
                // pub success: serde::de::IgnoredAny,
                // pub ttbList: serde::de::IgnoredAny,
            }

            impl #original_ident {
                async fn call<D: Serialize + Sync>(castgc: &str, data: &D) -> Result<#original_ident> {
                    let client = crate::api::xmu_service::jw::get_castgc_client(castgc);
                    let res_auth = client.get(#original_ident::APP_ENTRANCE).await?;

                    let resp = client.post(#original_ident::URL_DATA, data).await?.json().await?;
                    Ok(resp)
                }
            }
        },
    };

    TokenStream::from(expanded)
}

fn get_type_constraints(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(tp) => {
            let last_segment = tp.path.segments.last()?;
            let ident_str = last_segment.ident.to_string();

            match ident_str.as_str() {
                "LlmVec" | "Vec" => {
                    // 进入泛型内部：LlmVec<T>
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                            let inner_desc = get_type_constraints(inner_type)
                                .unwrap_or_else(|| "对应类型元素".into());
                            return Some(format!(
                                "多个重复的条目，每个条目格式为: <item>{}</item>",
                                inner_desc
                            ));
                        }
                    }
                    Some("列表格式内容".into())
                }
                "LlmBool" | "bool" => Some("布尔值 (true/false)".into()),
                "i64" | "u64" | "i32" | "u32" => Some("纯数字".into()),
                "String" => Some("纯文本内容".into()),
                "LlmOption" | "Option" => {
                    if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                            return get_type_constraints(inner_type);
                        }
                    }
                    Some("可选内容".into())
                }
                _ => None,
            }
        }
        _ => None,
    }
}

#[proc_macro_derive(LlmPrompt, attributes(prompt))]
pub fn derive_llm_prompt(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut schema_parts = Vec::new();

    if let Data::Struct(data) = &input.data {
        for field in &data.fields {
            let field_ident = field.ident.as_ref().unwrap();
            let field_name = field_ident.to_string();
            let field_type = &field.ty;

            // 将类型转为字符串进行匹配判断
            let type_str = quote!(#field_type).to_string().replace(" ", "");

            // 1. 提取 #[prompt("...")]
            let mut user_description = None;
            for attr in &field.attrs {
                if attr.path().is_ident("prompt") {
                    let _ = attr.parse_nested_meta(|meta| {
                        if let Ok(lit) = meta.input.parse::<syn::LitStr>() {
                            user_description = Some(lit.value());
                        }
                        Ok(())
                    });
                }
            }

            // 2. 根据类型生成基础约束说明
            let type_constraint = get_type_constraints(field_type);

            // 3. 严格性检查：如果既没有内置类型判断，直接报错
            if type_constraint.is_none() {
                let err_msg = format!(
                    "字段 `{}` 的类型 `{}` 未定义 Prompt 解释。请修改宏中的 get_type_constraints 函数确定模型的返回格式。",
                    field_name, type_str
                );
                return syn::Error::new_spanned(field_ident, err_msg)
                    .to_compile_error()
                    .into();
            }

            // 合并描述
            let final_desc = match (type_constraint, user_description) {
                (Some(tc), Some(ud)) => format!("类型提示:{}; 其他提示:{}", tc, ud),
                (Some(tc), None) => tc.to_string(),
                (None, Some(ud)) => ud,
                _ => unreachable!(),
            };

            schema_parts.push(format!(
                "<{field_name}>...</{field_name}>  <!-- {} -->",
                final_desc
            ));
        }
    }

    let root_tag = name.to_string();
    let schema = format!("<{0}>\n  {1}\n</{0}>", root_tag, schema_parts.join("\n  "));

    let expanded = quote! {
        impl LlmPrompt for #name {
            fn get_prompt_schema() -> &'static str {
                #schema
            }
            fn root_name() -> &'static str {
                #root_tag
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn lnt_get_api(args: TokenStream, input: TokenStream) -> TokenStream {
    // 1. 解析参数：#[lnt_get_api(ResponseType, "url")]
    let arg_parser = Punctuated::<Expr, Token![,]>::parse_terminated;
    let args = parse_macro_input!(args with arg_parser);

    if args.len() < 2 {
        panic!(
            "\n[Macros Error]: Expected at least 2 arguments: #[lnt_get_api(ResponseType, \"url\")]\n"
        );
    }

    let response_type = &args[0];
    let url_expr = &args[1];

    // 提取 URL 字符串并获取其原始 Span 用于错误定位和高亮
    let url_string = if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = url_expr
    {
        s.value()
    } else {
        panic!("\n[Macros Error]: The second argument must be a string literal (the URL).\n");
    };

    let mut fn_params = Vec::new();
    let mut call_args = Vec::new();
    let mut clean_url = url_string.clone();

    // 2. 解析占位符 {name:Type}

    for (start, _) in url_string.match_indices('{') {
        if let Some(rel_end) = url_string[start..].find('}') {
            let end = start + rel_end;
            let capture = &url_string[start + 1..end]; // 例如 "course_id:i64"

            if let Some((name_str, ty_str)) = capture.split_once(':') {
                let name = name_str.trim();
                let ty_s = ty_str.trim();

                // 编译期类型存在性检查：如果类型写错，编译器红线会画在宏参数上
                let ty: Type = syn::parse_str(ty_s).unwrap_or_else(|_| {
                    panic!("\n[Macros Error]: The type '{}' for parameter '{}' is not a valid Rust type.\n", ty_s, name);
                });

                let name_ident = format_ident!("{}", name, span = url_expr.span());
                fn_params.push(quote! { #name_ident: #ty });
                call_args.push(quote! { #name_ident });

                // 语法糖还原：将 {id:i64} 替换为 {id}，以便标准 format! 识别
                let from = format!("{}:{}", name_str, ty_str);
                clean_url = clean_url.replace(&from, name);
            } else {
                // 默认回退到 i64 (满足你的偏好)
                let name = capture.trim();
                let name_ident = format_ident!("{}", name, span = url_expr.span());
                fn_params.push(quote! { #name_ident: i64 });
                call_args.push(quote! { #name_ident });
            }
        }
    }

    // 3. 准备生成的字面量和结构体
    let clean_url_lit = LitStr::new(&clean_url, url_expr.span());
    let input_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &input_struct.ident;

    let url_builder = if call_args.is_empty() {
        // 情况 A：没有参数，直接转 String，避免 format! 损耗
        quote! { #clean_url_lit.to_string() }
    } else {
        // 情况 B：有参数，使用显式绑定 key = value 消除 redundant 警告并支持高亮
        quote! { format!(#clean_url_lit, #(#call_args = #call_args),*) }
    };

    // 4. 生成代码：关联 Span 以实现 IDE 高亮和精准报错
    let expanded = quote_spanned! { url_expr.span() =>
        #input_struct

        impl #struct_name {
            #[allow(dead_code)]
            pub async fn get(session: &str, #(#fn_params),*) -> anyhow::Result<#response_type> {
                let client = crate::api::xmu_service::lnt::get_session_client(session);
                Self::get_from_client(&client, #(#call_args),*).await
            }

            pub async fn get_from_client(client:&crate::api::network::SessionClient, #(#fn_params),*) -> anyhow::Result<#response_type> {
                // 2. 构造 URL (IDE 会在此处通过 Span 关联实现高亮)
                let target_url = #url_builder;

                // 3. 执行请求并处理分级英文错误
                let res = client.get(&target_url).await
                    .map_err(|e| anyhow::anyhow!("Network Error: Failed to reach '{}'. Details: {}", target_url, e))?;

                if !res.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "HTTP Error: API returned status {} for URL: {}",
                        res.status(),
                        target_url
                    ));
                }

                // 4. 反序列化
                let data = res.json::<#response_type>().await
                    .map_err(|e| anyhow::anyhow!(
                        "Deserialization Error: Failed to parse {} from {}. Error: {}",
                        stringify!(#response_type),
                        target_url,
                        e
                    ))?;

                Ok(data)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn session_client_helper(_args: TokenStream, input: TokenStream) -> TokenStream {
    // 1. 解析输入的函数
    let input_fn = parse_macro_input!(input as ItemFn);
    let sig = &input_fn.sig;
    let old_name = sig.ident.to_string();
    let suffix = "_from_client";

    // 2. 校验后缀：必须以 _from_client 结尾
    if !old_name.ends_with(suffix) {
        return TokenStream::from(quote_spanned! {
            sig.ident.span() => compile_error!("Function name must end with '_from_client' (e.g., 'get_from_client')");
        });
    }

    // 3. 生成新函数名：去掉 "_from_client"
    // 例如 "get_from_client" -> "get"
    let new_name_str = &old_name[..old_name.len() - suffix.len()];

    // 如果去掉后缀后名字为空（比如函数名就叫 _from_client），给个默认名或报错
    if new_name_str.is_empty() {
        return TokenStream::from(quote_spanned! {
            sig.ident.span() => compile_error!("Function name is too short after removing '_from_client'");
        });
    }

    let new_name = format_ident!("{}", new_name_str, span = sig.ident.span());

    // 4. 提取签名要素
    let vis = &input_fn.vis;
    let generics = &sig.generics;
    let where_clause = &generics.where_clause;
    let return_type = &sig.output;

    // 5. 处理参数：保留除第一个 (&SessionClient) 之外的所有参数
    let mut inputs_iter = sig.inputs.iter();
    let first_arg = inputs_iter.next();

    let is_arc = match first_arg {
        Some(FnArg::Typed(pat_type)) => match &*pat_type.ty {
            // 匹配 &SessionClient (引用类型)
            syn::Type::Reference(ty_ref) => {
                if let syn::Type::Path(tp) = &*ty_ref.elem {
                    // 检查路径最后一段是否是 SessionClient
                    let last_seg = tp.path.segments.last().unwrap();
                    if last_seg.ident == "SessionClient" {
                        false
                    } else {
                        return TokenStream::from(quote_spanned! {
                            pat_type.ty.span() => compile_error!("the type of client should be the 'SessionClient'");
                        });
                    }
                } else {
                    return TokenStream::from(quote_spanned! {
                        pat_type.ty.span() => compile_error!("the first argument must be '&SessionClient'");
                    });
                }
            }
            // 匹配 Arc<SessionClient> (路径类型)
            syn::Type::Path(ty_path) => {
                let last_seg = ty_path.path.segments.last().unwrap();
                if last_seg.ident == "Arc" {
                    // 进一步校验泛型参数是否为 SessionClient
                    let mut valid_inner = false;
                    if let syn::PathArguments::AngleBracketed(args) = &last_seg.arguments {
                        if let Some(syn::GenericArgument::Type(syn::Type::Path(inner_tp))) =
                            args.args.first()
                        {
                            if inner_tp.path.segments.last().map(|s| &s.ident)
                                == Some(&format_ident!("SessionClient"))
                            {
                                valid_inner = true;
                            }
                        }
                    }
                    if valid_inner {
                        true
                    } else {
                        return TokenStream::from(quote_spanned! {
                            pat_type.ty.span() => compile_error!("the type inside 'Arc' must be 'SessionClient'");
                        });
                    }
                } else {
                    return TokenStream::from(quote_spanned! {
                        pat_type.ty.span() => compile_error!("the first argument must be 'Arc<SessionClient>' or '&SessionClient'");
                    });
                }
            }
            _ => {
                return TokenStream::from(quote_spanned! {
                    pat_type.ty.span() => compile_error!("Unsupported type for client argument");
                });
            }
        },

        _ => {
            return TokenStream::from(quote_spanned! {
                sig.span() => compile_error!("Function must have at least one argument like: client: &SessionClient");
            });
        }
    };

    let other_params: Vec<_> = inputs_iter.collect();

    // 提取用于调用的参数标识符 (例如 prompt)
    let call_args: Vec<_> = other_params
        .iter()
        .map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    let id = &pat_ident.ident;
                    return quote! { #id };
                }
            }
            quote! { _ }
        })
        .collect();

    // 6. 生成新函数
    let client_invocation = if is_arc {
        // 如果原函数要 Arc，我们把 get_session_client 返回的引用包装成新的 Arc
        // 假设 get_session_client 返回的是 SessionClient 实例
        quote! { Arc::new(client) }
    } else {
        // 如果原函数只要引用，维持原状
        quote! { &client }
    };

    let original_ident = &sig.ident;
    let gen_fn = quote_spanned! { sig.span() =>
        #vis async fn #new_name #generics (
            session: &str,
            #(#other_params),*
        ) #return_type #where_clause {
            // 初始化 Client 并设置 Cookie
            let client = crate::api::xmu_service::lnt::get_session_client(session);

            // 内部调用原始的 _from_client 函数
            Self::#original_ident(#client_invocation, #(#call_args),*).await
        }
    };

    // 7. 合并输出：保留原函数，追加新函数
    let expanded = quote! {
        #input_fn
        #gen_fn
    };

    TokenStream::from(expanded)
}
