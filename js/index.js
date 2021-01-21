import("../pkg/index.js").then(module => {
    console.log(module);
    module.greet();
}).catch(console.error);
