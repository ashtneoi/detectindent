use count_indents;

#[test]
fn count1() {
    let lines = [
        "    x",
        "x",
        "x",
        "\t\tx",
        "        x",
    ].iter().map(|x| Ok(x.to_string()));
    assert_eq!(
        count_indents(lines).unwrap(),
        (true, vec![4, 8])
    );
}

#[test]
fn count2() {
    let lines = [
        "x",
        "x",
        "\tx",
        "\t\tx",
        "x",
    ].iter().map(|x| Ok(x.to_string()));
    assert_eq!(
        count_indents(lines).unwrap(),
        (true, vec![])
    );
}

#[test]
fn count3() {
    let lines = [
        "  x",
        "x",
        "x",
        "  x",
        "    x",
    ].iter().map(|x| Ok(x.to_string()));
    assert_eq!(
        count_indents(lines).unwrap(),
        (false, vec![2, 2, 4])
    );
}
