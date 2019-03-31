list = [true, false, 123, -123.456, "foobar"],
list_multi = [ # hmm
    true, # Comment
    false, // Test
    123, /* Comment */
    # "non-existent",
    /* Comment */ -123.456,
    "foobar",
]

list_in_list = [ // This should work
    [/* Inline start */ "test", "foobar"],
    1,
    2,
    -3,
]


map_in_list = [ /* This too! */
    {
        test = 123
    },
    {
        foo = "bar"
    },
    {
        baz = false,
    },
]
