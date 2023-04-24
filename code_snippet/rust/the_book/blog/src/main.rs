use blog::Post;

// this example implements State Pattern. (described in Gang-of-Four book)
// Note
// - you can also apply `enum` instead of `state object`, that requires
//   match expression spreading in several places of your app code.
// - downside of State Pattern :
//   - duplicate code in some of state objects (can be solved by `macro`)
//   - some of state objects may be coupled together (to make transition
//     work correctly)
fn main() {
    let sections = vec![
        "# Database ACID properties",
        "## Atomicity\nall or none of ops done in a transaction",
        "## Consistency\nstate change from one to another has to be consistent",
        "## Isolation\nconcurrent transactions executed sequentially",
        "## Duration\ndata persistence in storage device",
    ];
    let mut post = Post::new(2000u16);
    for section in sections.iter() {
        assert!(post.add_new_text(section));
        assert_eq!(post.content(), "");
    }
    assert!(post.request_review());
    assert_eq!(post.content(), "");
    assert!(post.approve());
    let actual_content = post.content();
    let expect_content = String::new();
    let expect_content = expect_content + &sections[0] + &sections[1] +
        &sections[2] + &sections[3] + &sections[4];
    assert_eq!(actual_content, expect_content);
    println!("final article : {actual_content}");
}
