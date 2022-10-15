trait WebLoop {
    fn run(host: impl AsRef<str>, port: u16);
}
