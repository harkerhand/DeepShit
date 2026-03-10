use jrecord_derive::ai_logic;

#[ai_logic("生成一个 validate(&self) -> bool 方法，检查 id 是否不为空且 value 长度大于 5")]
struct UserRecord {
    id: String,
    name: String,
    value: String,
}

fn main() {
    let user = UserRecord {
        id: "1".into(),
        name: "John Doe".into(),
        value: "Some value".into(),
    };

    println!("ID: {}", user.id);
    println!("Name: {}", user.name);
    println!("Value: {}", user.value);

    // 这个 validate 方法是编译期由 AI 生成并注入到代码里的！
    if user.validate() {
        println!("Record is valid!");
    }
}
