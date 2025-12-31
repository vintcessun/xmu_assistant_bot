extern crate proc_macro;
use darling::FromMeta;
use heck::AsPascalCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Error, Ident, ItemFn, ItemStruct, LitBool, LitStr, Meta, Path, Result, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
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
                let msg_body = ctx.get_message();
                let msg = match &*msg_body {
                    Message::Group(g) => &g.message,
                    Message::Private(p) => &p.message,
                };
                ctx.send_message_async(message::receive2send_add_prefix(
                    msg,
                    match ctx.get_target() {
                        Target::Group(group_id) => format!(
                            "来自群({group_id})的{}({} {})命令: ",
                            ctx.sender
                                .card
                                .as_ref()
                                .unwrap_or(&String::from("未知群昵称")),
                            &ctx.sender
                                .nickname
                                .as_ref()
                                .unwrap_or(&String::from("未知昵称")),
                            &ctx.sender.user_id.unwrap_or(0),
                        ),
                        Target::Private(user_id) => {
                            format!(
                                "用户{user_id}({})的命令: ",
                                &ctx.sender
                                    .nickname
                                    .as_ref()
                                    .unwrap_or(&String::from("未知昵称"))
                            )
                        }
                    },
                ));
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
            let plugin_logic = |mut ctx: Context<T, #target_type_ident>| async move {
                let result: anyhow::Result<()> = { #body };
                result
            };
            plugin_logic(ctx).await
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
                        tracing::error!("Handler [{}] 运行出错: {:?}", stringify!(#handler), e);
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
struct JwApiArgs {
    url: String,
    app: String,
    field_name: String,
}

impl Parse for JwApiArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vars = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        let mut url: Option<String> = None;
        let mut app: Option<String> = None;

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

        Ok(JwApiArgs {
            url,
            app,
            field_name,
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

    let vis = &input_struct.vis;
    let fields = &input_struct.fields;
    let url_val = args.url;
    let app_val = args.app;

    let field_name_from_url = args.field_name;

    let dynamic_field_ident = format_ident!("{}", field_name_from_url);

    let expanded = quote! {
        #[derive(Serialize, Deserialize, Debug, Clone, Default)]
        #[serde(rename_all = "UPPERCASE")]
        #vis struct #response_item_ident
        #fields

        #[async_trait::async_trait]
        impl JwAPI for #original_ident {
            const URL_DATA: &'static str = #url_val;
            const APP_ENTRANCE: &'static str = #app_val;
            type Response = #response_item_ident;
        }

        #[derive(Serialize, Deserialize, Debug, Clone, Default)]
        #[serde(rename_all = "camelCase")]
        #vis struct #data_api_ident {
            pub rows: Vec<#response_item_ident>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub ext_params: Option<Box<serde_json::value::RawValue>>,
            pub page_number: Option<Box<serde_json::value::RawValue>>,
            pub page_size: Option<Box<serde_json::value::RawValue>>,
            pub total_size: Option<Box<serde_json::value::RawValue>>,
        }

        #[derive(Serialize, Deserialize, Debug, Clone, Default)]
        #vis struct #datas_ident {
            pub #dynamic_field_ident: #data_api_ident,
        }

        #[derive(Serialize, Deserialize, Debug, Clone, Default)]
        #vis struct #original_ident {
            pub code: String,
            pub datas: #datas_ident,
        }

        impl #original_ident {
            async fn call<D: Serialize + Sync>(castgc: &str, data: &D) -> Result<#original_ident> {
                let mut client = SessionClient::new();
                client.set_cookie(
                    "CASTGC",
                    castgc,
                    IDS_URL.clone(),
                );
                let res_auth = client.get(#original_ident::APP_ENTRANCE).await?;

                let resp = client.post(#original_ident::URL_DATA, data).await?.json().await?;
                Ok(resp)
            }
        }
    };

    TokenStream::from(expanded)
}
