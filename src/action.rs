struct Login;

impl Action for Login {
    fn execute(httpExchange: &HttpExchange) {}
}

trait Action {
    fn execute(httpExchange: &HttpExchange);
}
