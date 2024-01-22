use serde::Deserialize;

use serde_json::from_str;

pub fn test<'de, T:Deserialize<'de>>(test: &'de str) -> T {
    let r = from_str::<'de,T>(test);
    r.unwrap()
}

#[cfg(test)]
mod tests {

    use super::*;


    #[derive(Debug,PartialEq, Eq,Deserialize)]
    struct Test{
        a: i32,
        b: bool,
        c: String
    }

    #[test]
    fn it_works() {

        let j = r#"{
            "a": 123,
            "b": true,
            "c": "test"
        }"#;

        let result: Test = test(j);
        println!("{:?}",&result);

        let test = Test {
            a:123,
            b:true,
            c:"test".to_owned()
        };
        assert_eq!(result, test);
    }
}
