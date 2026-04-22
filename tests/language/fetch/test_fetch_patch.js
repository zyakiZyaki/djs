// fetch() with PATCH method
function test() {
    function handleResponse(res) {
        return res.status;
    }
    return fetch("https://httpbin.org/patch", {
        method: "PATCH",
        body: { title: "test", body: "hello" }
    }).then(handleResponse);
}
test()