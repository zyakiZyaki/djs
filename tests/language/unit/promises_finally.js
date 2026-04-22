new Promise(function(resolve) { resolve(1); }).finally(function() { return 99; }).then(function(x) { return x; })
