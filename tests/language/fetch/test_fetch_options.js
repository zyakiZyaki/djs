// fetch() with OPTIONS method
function test() {
    function handleResponse(res) {
        return res.status;
    }
    return fetch("https://httpbin.org/get", {
        method: "OPTIONS"
    }).then(handleResponse);
}
test()