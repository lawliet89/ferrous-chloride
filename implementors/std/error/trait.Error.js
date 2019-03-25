(function() {var implementors = {};
implementors["failure"] = [{text:"impl&lt;E:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> for <a class=\"struct\" href=\"failure/struct.Compat.html\" title=\"struct failure::Compat\">Compat</a>&lt;E&gt;",synthetic:false,types:["failure::compat::Compat"]},];
implementors["nom"] = [{text:"impl&lt;I, E&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> for <a class=\"enum\" href=\"nom/enum.Err.html\" title=\"enum nom::Err\">Err</a>&lt;I, E&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;I: <a class=\"trait\" href=\"nom/lib/std/fmt/trait.Debug.html\" title=\"trait nom::lib::std::fmt::Debug\">Debug</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;E: <a class=\"trait\" href=\"nom/lib/std/fmt/trait.Debug.html\" title=\"trait nom::lib::std::fmt::Debug\">Debug</a>,&nbsp;</span>",synthetic:false,types:["nom::internal::Err"]},];
implementors["syn"] = [{text:"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> for <a class=\"struct\" href=\"syn/struct.Error.html\" title=\"struct syn::Error\">Error</a>",synthetic:false,types:["syn::error::Error"]},];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
