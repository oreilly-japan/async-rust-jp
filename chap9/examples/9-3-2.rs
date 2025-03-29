trait Greeting {
    fn greet(&self) -> String;
}

struct HelloWorld;
impl Greeting for HelloWorld {
    fn greet(&self) -> String {
        "Hello, World!".to_string()
    }
}

struct ExcitedGreeting<T> {
    inner: T,
}

impl<T> Greeting for ExcitedGreeting<T> where T: Greeting{
    fn greet(&self) -> String {
        let mut greeting = self.inner.greet();
        greeting.push_str(" I'm so excited to be in Rust!");
        greeting
    }
}

fn main() {
    #[cfg(feature = "logging_decorator")]
    let hello = ExcitedGreeting { inner: HelloWorld };

    #[cfg(not(feature = "logging_decorator"))]
    let hello = HelloWorld;

    println!("{}", hello.greet());
}