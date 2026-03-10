extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use reqwest::blocking::Client;
use serde_json::json;
use syn::{ItemStruct, LitStr, parse_macro_input};

#[proc_macro_attribute]
pub fn ai_logic(args: TokenStream, input: TokenStream) -> TokenStream {
    // 解析指令字符串
    let instruction = parse_macro_input!(args as LitStr).value();
    // 解析结构体
    let item_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &item_struct.ident;

    // 打印当前正在处理的结构体，方便在编译终端查看进度
    println!("---------- AI Code Gen: {} ----------", struct_name);

    let fields_info = item_struct
        .fields
        .iter()
        .map(|f| format!("{}: {:?}", f.ident.as_ref().unwrap(), f.ty))
        .collect::<Vec<_>>()
        .join(", ");

    let prompt = format!(
        "You are a Rust compiler. For: struct {} {{ {} }}\n\
         Task: {}\n\
         Output ONLY the 'impl {} {{ ... }}' block. NO markdown, NO text, NO comments.",
        struct_name, fields_info, instruction, struct_name
    );

    // 直接请求，不走缓存
    let ai_generated_code = match call_llm_api(&prompt) {
        Ok(raw) => {
            let cleaned = clean_code_block(&raw);
            // 这里用 eprintln 是为了让信息直接在 cargo build 的终端里显眼地跳出来
            eprintln!(
                "AI Response for {}:\n{}\n--------------------",
                struct_name, cleaned
            );
            cleaned
        }
        Err(e) => {
            eprintln!("LLM API Error: {}", e);
            String::new()
        }
    };

    // 解析生成的代码。如果失败，输出 compile_error 让开发者在 IDE 里能看到报错
    let ai_tokens: proc_macro2::TokenStream = ai_generated_code.parse().unwrap_or_else(|_| {
        let err = format!(
            "AI output could not be parsed as Rust: {}",
            ai_generated_code
        );
        quote! { compile_error!(#err); }
    });

    // 组合：必须把原有的 #item_struct 放回去，否则结构体定义就丢了
    let expanded = quote! {
        #item_struct

        #ai_tokens
    };

    TokenStream::from(expanded)
}

fn clean_code_block(input: &str) -> String {
    // 移除可能存在的 Markdown 标签
    let mut output = input
        .replace("```rust", "")
        .replace("```", "")
        .replace("`", ""); // 有时候 AI 喜欢用单个反引号包裹

    // 只保留从 impl 开始的部分，过滤掉前面的废话
    if let Some(start) = output.find("impl") {
        output = output[start..].to_string();
    }
    output.trim().to_string()
}

fn call_llm_api(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 从环境变量获取 API_KEY
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .map_err(|_| "DEEPSEEK_API_KEY environment variable not set".to_string())?;

    // 增加超时，防止 API 响应慢导致编译卡死
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()?;

    let res = client
        .post("https://api.deepseek.com/v1/chat/completions") // 修正了 URL
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "deepseek-chat",
            "messages": [
                {"role": "system", "content": "You are a specialized Rust code generator. You output ONLY valid Rust syntax without any natural language."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.0
        }))
        .send()?
        .json::<serde_json::Value>()?;

    let content = res["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in AI response")?;

    Ok(content.to_string())
}
