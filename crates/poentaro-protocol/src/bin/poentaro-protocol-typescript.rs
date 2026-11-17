//! Writes each audience's generated TypeScript client into the folder given
//! as the only argument, as `<audience>.ts`. The web app runs it before it
//! builds (`build:protocol` in `web/package.json`).

const CLIENTS: [(&str, &str); 3] = [
    ("app", include_str!(concat!(env!("OUT_DIR"), "/app.ts"))),
    (
        "public",
        include_str!(concat!(env!("OUT_DIR"), "/public.ts")),
    ),
    ("test", include_str!(concat!(env!("OUT_DIR"), "/test.ts"))),
];

fn main() {
    let folder = std::env::args_os()
        .nth(1)
        .expect("usage: poentaro-protocol-typescript <folder>");
    let folder = std::path::Path::new(&folder);
    std::fs::create_dir_all(folder).expect("the folder can be made");
    for (audience, client) in CLIENTS {
        std::fs::write(folder.join(format!("{audience}.ts")), client)
            .unwrap_or_else(|error| panic!("{audience}.ts: {error}"));
    }
}
