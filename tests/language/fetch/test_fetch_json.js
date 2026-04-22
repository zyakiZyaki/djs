// Test for fetch with JSON response
function test() {
    function handleResponse(res) {
        return res.json();
    }
    
    function handleError(err) {
        return { error: "failed" };
    }
    
    // Test with a real site that returns JSON data
    return fetch("https://httpbin.org/json").then(handleResponse, handleError);
}

test()