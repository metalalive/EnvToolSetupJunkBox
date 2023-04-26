// A trait with a generic type parameter
trait ExampleTrait<T,U> {
    fn do_something(&self, value: T) -> U;
}

// Any trait with generic type can be implemented with any struct
// type and duplicate concrete types

// A struct that implements ExampleTrait with a concrete type `i8`
struct IntBundle {value: i8,}
impl ExampleTrait<i8,i16> for IntBundle {
    fn do_something(&self, v: i8) -> i16 {
        (self.value as i16) * (v as i16)
    }
}
impl ExampleTrait<i8,i8> for IntBundle {
    fn do_something(&self, v: i8) -> i8 {
        self.value + v
    }
}
impl ExampleTrait<f32,f64> for IntBundle {
    fn do_something(&self, v: f32) -> f64 {
        (self.value as f64) * (v as f64)
    }
}

// A trait with an associated type
trait AnotherTrait {
    type Output;    
    fn do_something_else(&self) -> Self::Output;
}

// A struct that implements AnotherTrait with a concrete type for Output
struct AnotherStruct {value: u32,}
impl AnotherTrait for AnotherStruct {
    type Output = String;    
    fn do_something_else(&self) -> Self::Output {
        format!("The value in AnotherStruct is: {}", self.value)
    }
}

// struct DupStrStruct {value: String,}

// the code below won't compile cuz it duplicates the implementation
// of the struct `AnotherStruct`  with the same associated type.
// As mentioned in The Book, associated type within a struct is part
// of struct signature
// impl AnotherTrait for AnotherStruct {
//     type Output = String;
//     fn do_something_else(&self) -> Self::Output {
//         format!("The value is: {} {}", self.value, self.value)
//     }
// }

fn main() {
    let example = IntBundle { value: 42 };
    let result:i16 = example.do_something(-10);
    println!("The integer is: {result}");
    let result:i8 = example.do_something(-2);
    println!("The integer(two) is: {result}");
    let result:f64 = example.do_something(2.7);
    println!("The floating-point number is: {result}");
    
    let another = AnotherStruct { value: 43 };
    let result = another.do_something_else();
    println!("{result}");
    // let another = DupStrStruct { value:"snake master".to_string() };
    // let result = another.do_something_else();
    // println!("{result}");
}

