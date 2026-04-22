// fetch() with HEAD method
function test() {
    function handleResponse(res) {
        return res.status;
    }
    return fetch("https://httpbin.org/get", {
        method: "HEAD"
    }).then(handleResponse);
}
test()