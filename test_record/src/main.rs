use jrecord_derive::{ai_impl, ai_struct};

// 场景 A：直接在结构体上派生复杂逻辑
#[ai_struct("生成一个 new 方法以及所有的 getter")]
pub struct User {
    id: u64,
    username: String,
    email: String,
}

struct Calculator {
    precision: u32,
}

#[ai_impl("实现四则运算方法，计算结果保留 precision 指定的小数位数, precision可以从self获得")]
impl Calculator {}

fn main() {
    // 场景 A 的调用
    let user = User::new(1, "DeepSeeker".into(), "ai@example.com".into());
    println!("User: {}", user.username());

    // 场景 B 的调用
    let calc = Calculator { precision: 2 };
    println!("{}", calc.add(1.1, 2.2));
}
