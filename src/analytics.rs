
use serde_json;


#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct BotDectectorStats {
    active_conns: u32,
}


#[test]
fn test_json() {
    let d1 = BotDectectorStats {
        active_conns: 1,
    };
    let s = serde_json::to_string(&d1).unwrap();
    let d2: BotDectectorStats = serde_json::from_str(&s).unwrap();
    assert_eq!(d1, d2);
}
