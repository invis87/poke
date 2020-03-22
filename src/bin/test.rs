#[derive(Debug)]
struct NotCopyable {
    x: i32
}
#[derive(Debug)]
struct TwoFields {
    first: NotCopyable,
    second: NotCopyable,
}

pub fn main() {
    let two_fields = TwoFields {
        first: NotCopyable {x: 22} ,
        second: NotCopyable {x: 44} ,

    };


    let mut vector_of_tuples: Vec<(NotCopyable, NotCopyable)> = Vec::new();

   match two_fields {
       TwoFields {first: f, second: s} => vector_of_tuples.push((f, s)),
   } ;

    println!("{:?}", vector_of_tuples);
}


