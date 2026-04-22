// ERROR: handler is a global function, not a closure
function handler(req) {
    return { status: 200, headers: {}, body: "fail" };
}
function server() {
    return http.createServer(handler).listen(8888);
}
server()
