extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use reqwest::blocking::Client;
use serde_json::json;
use syn::{ItemImpl, ItemStruct, LitStr, parse_macro_input};

// --- 1. 处理结构体的属性宏 ---
#[proc_macro_attribute]
pub fn ai_struct(args: TokenStream, input: TokenStream) -> TokenStream {
    let instruction = parse_macro_input!(args as LitStr).value();
    let item_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &item_struct.ident;

    // 提取字段信息，给 AI 提供最准确的上下文
    let fields_info = item_struct
        .fields
        .iter()
        .map(|f| format!("{}: {:?}", f.ident.as_ref().unwrap(), f.ty))
        .collect::<Vec<_>>()
        .join(", ");

    let prompt = format!(
        "Context: Rust Struct Definition.\nSource: struct {} {{ {} }}\nTask: {}\n\
         Output ONLY an 'impl {} {{ ... }}' block. No markdown, no talk.",
        struct_name, fields_info, instruction, struct_name
    );

    let ai_code = call_llm_api(&prompt);
    let ai_tokens = parse_ai_code(&ai_code);

    quote! {
        #item_struct
        #ai_tokens
    }
    .into()
}

// --- 2. 处理 impl 块的属性宏 ---
#[proc_macro_attribute]
pub fn ai_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let instruction = parse_macro_input!(args as LitStr).value();
    let item_impl = parse_macro_input!(input as ItemImpl);
    let self_ty = &item_impl.self_ty; // 拿到 impl 后的类型名

    // 对于 impl 块，AI 往往不知道字段，所以 Prompt 需要稍微调整
    let prompt = format!(
        "Context: Rust impl block for type {:?}.\nTask: {}\n\
         Generate the methods inside the impl block. Return ONLY the complete 'impl ... {{ ... }}' block.",
        self_ty, instruction
    );

    let ai_code = call_llm_api(&prompt);
    let ai_tokens = parse_ai_code(&ai_code);

    // 注意：这里我们通常会替换掉原本空的 impl 块
    quote! {
        #ai_tokens
    }
    .into()
}

// --- 通用辅助函数 ---

fn call_llm_api(prompt: &str) -> String {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .unwrap();

    let base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "https://api.deepseek.com".to_string());

    let api_key = std::env::var("API_KEY").unwrap();

    let model = std::env::var("MODEL").unwrap_or_else(|_| "deepseek-chat".to_string());

    let res = client.post(format!("{}/v1/chat/completions", base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": model,
            "messages": [
                {"role": "system", "content": "You are a Rust expert. You only output valid Rust code blocks."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.0
        })).send();

    match res {
        Ok(response) => {
            let json: serde_json::Value = response.json().unwrap_or_default();
            json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string()
        }
        Err(e) => {
            eprintln!("API Error: {}", e);
            String::new()
        }
    }
}

fn parse_ai_code(raw: &str) -> proc_macro2::TokenStream {
    let mut cleaned = raw.replace("```rust", "").replace("```", "");
    if let Some(start) = cleaned.find("impl") {
        cleaned = cleaned[start..].to_string();
    }
    cleaned.parse().unwrap_or_else(|_| {
        let err = format!("AI output parse error: {}", cleaned);
        quote! { compile_error!(#err); }
    })
}
