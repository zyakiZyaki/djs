// http.delete() → DELETE /delete
function test() {
    function handleResponse(res) {
        return res.status;
    }
    return http.delete("http://localhost:8888/delete").then(handleResponse);
}
test()
