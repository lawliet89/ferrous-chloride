var N = null;var searchIndex = {};
searchIndex["fecl"]={"doc":"","items":[],"paths":[]};
searchIndex["ferrous_chloride"]={"doc":"","items":[[4,"Error","ferrous_chloride","Error type for this library",N,N],[13,"InvalidUnicodeCodePoint","","",0,N],[13,"InvalidNumber","","",0,N],[13,"InvalidUnicode","","",0,N],[13,"ParseError","","",0,N],[13,"IllegalMultipleEntries","","",0,N],[12,"key","ferrous_chloride::Error","",0,N],[12,"variant","","",0,N],[13,"ErrorMergingKeys","ferrous_chloride","",0,N],[12,"key","ferrous_chloride::Error","",0,N],[12,"existing_variant","","",0,N],[12,"incoming_variant","","",0,N],[13,"UnexpectedVariant","ferrous_chloride","",0,N],[12,"enum_type","ferrous_chloride::Error","",0,N],[12,"expected","","",0,N],[12,"actual","","",0,N],[4,"OneOrMany","ferrous_chloride","Either a single value, or many values",N,N],[13,"One","","",1,N],[13,"Many","","",1,N],[4,"KeyValuePairs","","A set of `(Key, Value)` pairs which can exist in a merged…",N,N],[13,"Merged","","",2,N],[13,"Unmerged","","",2,N],[11,"from_err_bytes","","Convert a Nom Err into something useful",0,[[["err"]],["self"]]],[11,"from_err_str","","Convert a Nom Err into something useful",0,[[["err"]],["self"]]],[11,"make_custom_error","","Convert to a Custom Nom Error",0,[[["err"],["f"]],["err",["error"]]]],[11,"make_custom_err_str","","",0,[[["err"]],["err",["error"]]]],[11,"make_custom_err_bytes","","",0,[[["err"]],["err",["error"]]]],[0,"iter","","Iterator Types and implementations for data structures In…",N,N],[4,"OneOrManyIterator","ferrous_chloride::iter","",N,N],[13,"One","","",3,N],[13,"Many","","",3,N],[4,"OneOrManyIntoIterator","","",N,N],[13,"One","","",4,N],[13,"Many","","",4,N],[4,"KeyValuePairsIterator","","",N,N],[13,"Merged","","",5,N],[13,"Unmerged","","",5,N],[4,"KeyValuePairsIntoIterator","","",N,N],[13,"Merged","","",6,N],[13,"Unmerged","","",6,N],[4,"KeyIterator","","",N,N],[13,"Merged","","",7,N],[13,"Unmerged","","",7,N],[4,"ValueIterator","","",N,N],[13,"Merged","","",8,N],[13,"Unmerged","","",8,N],[0,"literals","ferrous_chloride","Tokens and literals",N,N],[4,"Key","ferrous_chloride::literals","A \"key\" in a map",N,N],[13,"Identifier","","",9,N],[13,"String","","",9,N],[4,"Number","","Parsed Number",N,N],[13,"Integer","","",10,N],[13,"Float","","",10,N],[5,"boolean","","",N,[[["completestr"]],["iresult",["completestr","bool","u32"]]]],[5,"identifier","","",N,[[["completestr"]],["iresult",["completestr","str","u32"]]]],[5,"key","","",N,[[["completestr"]],["iresult",["completestr","key","u32"]]]],[5,"number","","",N,[[["completestr"]],["iresult",["completestr","number","u32"]]]],[5,"quoted_single_line_string","","",N,[[["completestr"]],["iresult",["completestr","string","u32"]]]],[5,"string","","",N,[[["completestr"]],["iresult",["completestr","string","u32"]]]],[11,"new_identifier","","",9,[[["str"]],["self"]]],[11,"new_identifier_owned","","",9,[[["string"]],["self"]]],[11,"new_string","","",9,[[["str"]],["self"]]],[11,"new_string_owned","","",9,[[["string"]],["self"]]],[11,"unwrap","","",9,[[["self"]],["cow",["str"]]]],[0,"value","ferrous_chloride","",N,N],[4,"Value","ferrous_chloride::value","Value in HCL",N,N],[13,"Integer","","",11,N],[13,"Float","","",11,N],[13,"Boolean","","",11,N],[13,"String","","",11,N],[13,"List","","",11,N],[13,"Map","","",11,N],[13,"Block","","",11,N],[5,"list","","",N,[[["completestr"]],["iresult",["completestr","vec","u32"]]]],[5,"single_value","","",N,[[["completestr"]],["iresult",["completestr","value","u32"]]]],[5,"key_value","","",N,[[["completestr"]],["iresult",["completestr","u32"]]]],[5,"map_values","","",N,[[["completestr"]],["iresult",["completestr","mapvalues","u32"]]]],[6,"Block","","",N,N],[6,"Map","","",N,N],[6,"MapValues","","",N,N],[6,"List","","",N,N],[11,"new_list","","",11,[[["t"]],["self"]]],[11,"new_map","","",11,[[["i"]],["self"]]],[11,"new_single_map","","",11,[[["t"]],["self"]]],[11,"new_block","","",11,N],[11,"variant_name","","",11,[[["self"]],["str"]]],[11,"is_scalar","","",11,[[["self"]],["bool"]]],[11,"is_aggregate","","",11,[[["self"]],["bool"]]],[11,"len","","\"Top\" level length",11,[[["self"]],["usize"]]],[11,"is_empty","","Whether Value is empty",11,[[["self"]],["bool"]]],[11,"integer","","",11,[[["self"]],["result",["i64","error"]]]],[11,"unwrap_integer","","Panics Panics if the variant is not an integer",11,[[["self"]],["i64"]]],[11,"float","","",11,[[["self"]],["result",["f64","error"]]]],[11,"unwrap_float","","Panics Panics if the variant is not a float",11,[[["self"]],["f64"]]],[11,"boolean","","",11,[[["self"]],["result",["bool","error"]]]],[11,"unwrap_boolean","","Panics Panics if the variant is not a boolean",11,[[["self"]],["bool"]]],[11,"borrow_str","","",11,[[["self"]],["result",["str","error"]]]],[11,"unwrap_borrow_str","","Panics Panics if the variant is not a string",11,[[["self"]],["str"]]],[11,"borrow_string_mut","","",11,[[["self"]],["result",["string","error"]]]],[11,"unwrap_borrow_string_mut","","Panics Panics if the variant is not a string",11,[[["self"]],["string"]]],[11,"string","","",11,[[["self"]],["result",["string"]]]],[11,"unwrap_string","","Panics Panics if the variant is not a string",11,[[["self"]],["string"]]],[11,"borrow_list","","",11,[[["self"]],["result",["list","error"]]]],[11,"unwrap_borrow_list","","Panics Panics if the variant is not a string",11,[[["self"]],["list"]]],[11,"borrow_list_mut","","",11,[[["self"]],["result",["list","error"]]]],[11,"unwrap_borrow_list_mut","","Panics Panics if the variant is not a list",11,[[["self"]],["list"]]],[11,"list","","",11,[[["self"]],["result",["list"]]]],[11,"unwrap_list","","Panics Panics if the variant is not a list",11,[[["self"]],["list"]]],[11,"borrow_map","","",11,[[["self"]],["result",["map","error"]]]],[11,"unwrap_borrow_map","","Panics Panics if the variant is not a string",11,[[["self"]],["map"]]],[11,"borrow_map_mut","","",11,[[["self"]],["result",["map","error"]]]],[11,"unwrap_borrow_map_mut","","Panics Panics if the variant is not a map",11,[[["self"]],["map"]]],[11,"map","","",11,[[["self"]],["result",["map"]]]],[11,"unwrap_map","","Panics Panics if the variant is not a map",11,[[["self"]],["map"]]],[11,"borrow_block","","",11,[[["self"]],["result",["block","error"]]]],[11,"unwrap_borrow_block","","Panics Panics if the variant is not a string",11,[[["self"]],["block"]]],[11,"borrow_block_mut","","",11,[[["self"]],["result",["block","error"]]]],[11,"unwrap_borrow_block_mut","","Panics Panics if the variant is not a block",11,[[["self"]],["block"]]],[11,"block","","",11,[[["self"]],["result",["block"]]]],[11,"unwrap_block","","Panics Panics if the variant is not a block",11,[[["self"]],["block"]]],[11,"merge","","Recursively merge value",11,[[["self"]],["result",["error"]]]],[11,"new_merged","","",12,[[["t"]],["result",["error"]]]],[11,"new_unmerged","","",12,[[["t"]],["self"]]],[11,"merge","","",12,[[["self"]],["result",["error"]]]],[11,"as_merged","","",12,[[["self"]],["result",["error"]]]],[11,"unmerge","","",12,[[["self"]],["self"]]],[11,"as_unmerged","","",12,[[["self"]],["self"]]],[11,"borrow_keys","","Borrow the keys as `Vec<&str>` for more ergonomic indexing.",12,[[["self"]],["keyvaluepairs",["vec","mapvalues"]]]],[11,"new_merged","","",13,[[["t"]],["result",["error"]]]],[11,"new_unmerged","","",13,[[["t"]],["self"]]],[11,"merge","","",13,[[["self"]],["result",["error"]]]],[11,"as_merged","","",13,[[["self"]],["result",["error"]]]],[11,"unmerge","","",13,[[["self"]],["self"]]],[11,"as_unmerged","","",13,[[["self"]],["self"]]],[7,"INTEGER","ferrous_chloride","",N,N],[7,"FLOAT","","",N,N],[7,"BOOLEAN","","",N,N],[7,"STRING","","",N,N],[7,"LIST","","",N,N],[7,"MAP","","",N,N],[7,"BLOCK","","",N,N],[7,"MERGED","","",N,N],[7,"UNMERGED","","",N,N],[7,"VALUE","","",N,N],[7,"MAP_VALUES","","",N,N],[8,"ScalarLength","","Has scalar length",N,N],[10,"len_scalar","","Recursively count the number of scalars",14,[[["self"]],["usize"]]],[11,"is_empty_scalar","","",14,[[["self"]],["bool"]]],[8,"Mergeable","","Type is mergeable",N,N],[10,"is_merged","","Recursively checks that self is merged",15,[[["self"]],["bool"]]],[11,"is_unmerged","","Recursively checks that self is unmerged",15,[[["self"]],["bool"]]],[11,"len","","",1,[[["self"]],["usize"]]],[11,"is_empty","","",1,[[["self"]],["bool"]]],[11,"is_one","","",1,[[["self"]],["bool"]]],[11,"is_many","","",1,[[["self"]],["bool"]]],[11,"iter","","",1,[[["self"]],["oneormanyiterator"]]],[11,"unwrap_one","","",1,[[["self"]],["t"]]],[11,"unwrap_many","","",1,[[["self"]],["vec"]]],[11,"len","","",2,[[["self"]],["usize"]]],[11,"is_empty","","",2,[[["self"]],["bool"]]],[11,"unwrap_merged","","",2,[[["self"]],["hashmap"]]],[11,"unwrap_unmerged","","",2,[[["self"]],["vec"]]],[11,"iter","","",2,[[["self"]],["keyvaluepairsiterator"]]],[11,"keys","","",2,[[["self"]],["keyiterator"]]],[11,"values","","",2,[[["self"]],["valueiterator"]]],[11,"get_single","","Get a single value with the specified key.",2,[[["self"],["q"]],["option"]]],[11,"get","","",2,[[["self"],["q"]],["option",["oneormany"]]]],[14,"space_tab","","",N,N],[14,"map_err_str","","`map_err_str(IResult<I, O, u32>) -> IResult<I, O, Error>`",N,N],[14,"map_err","","`map_err_str(IResult<I, O, u32>) -> IResult<I, O, Error>`",N,N],[11,"to_string","","",0,[[["self"]],["string"]]],[11,"from","","",0,[[["t"]],["t"]]],[11,"into","","",0,[[["self"]],["u"]]],[11,"try_from","","",0,[[["u"]],["result"]]],[11,"borrow","","",0,[[["self"]],["t"]]],[11,"borrow_mut","","",0,[[["self"]],["t"]]],[11,"try_into","","",0,[[["self"]],["result"]]],[11,"get_type_id","","",0,[[["self"]],["typeid"]]],[11,"as_fail","","",0,[[["self"]],["fail"]]],[11,"into_iter","","",1,[[["self"]],["i"]]],[11,"from","","",1,[[["t"]],["t"]]],[11,"into","","",1,[[["self"]],["u"]]],[11,"to_owned","","",1,[[["self"]],["t"]]],[11,"clone_into","","",1,N],[11,"try_from","","",1,[[["u"]],["result"]]],[11,"borrow","","",1,[[["self"]],["t"]]],[11,"borrow_mut","","",1,[[["self"]],["t"]]],[11,"try_into","","",1,[[["self"]],["result"]]],[11,"get_type_id","","",1,[[["self"]],["typeid"]]],[11,"into_iter","","",2,[[["self"]],["i"]]],[11,"from","","",2,[[["t"]],["t"]]],[11,"into","","",2,[[["self"]],["u"]]],[11,"to_owned","","",2,[[["self"]],["t"]]],[11,"clone_into","","",2,N],[11,"try_from","","",2,[[["u"]],["result"]]],[11,"borrow","","",2,[[["self"]],["t"]]],[11,"borrow_mut","","",2,[[["self"]],["t"]]],[11,"try_into","","",2,[[["self"]],["result"]]],[11,"get_type_id","","",2,[[["self"]],["typeid"]]],[11,"into_iter","ferrous_chloride::iter","",3,[[["self"]],["i"]]],[11,"from","","",3,[[["t"]],["t"]]],[11,"into","","",3,[[["self"]],["u"]]],[11,"try_from","","",3,[[["u"]],["result"]]],[11,"borrow","","",3,[[["self"]],["t"]]],[11,"borrow_mut","","",3,[[["self"]],["t"]]],[11,"try_into","","",3,[[["self"]],["result"]]],[11,"get_type_id","","",3,[[["self"]],["typeid"]]],[11,"into_iter","","",4,[[["self"]],["i"]]],[11,"from","","",4,[[["t"]],["t"]]],[11,"into","","",4,[[["self"]],["u"]]],[11,"try_from","","",4,[[["u"]],["result"]]],[11,"borrow","","",4,[[["self"]],["t"]]],[11,"borrow_mut","","",4,[[["self"]],["t"]]],[11,"try_into","","",4,[[["self"]],["result"]]],[11,"get_type_id","","",4,[[["self"]],["typeid"]]],[11,"into_iter","","",5,[[["self"]],["i"]]],[11,"from","","",5,[[["t"]],["t"]]],[11,"into","","",5,[[["self"]],["u"]]],[11,"try_from","","",5,[[["u"]],["result"]]],[11,"borrow","","",5,[[["self"]],["t"]]],[11,"borrow_mut","","",5,[[["self"]],["t"]]],[11,"try_into","","",5,[[["self"]],["result"]]],[11,"get_type_id","","",5,[[["self"]],["typeid"]]],[11,"into_iter","","",6,[[["self"]],["i"]]],[11,"from","","",6,[[["t"]],["t"]]],[11,"into","","",6,[[["self"]],["u"]]],[11,"try_from","","",6,[[["u"]],["result"]]],[11,"borrow","","",6,[[["self"]],["t"]]],[11,"borrow_mut","","",6,[[["self"]],["t"]]],[11,"try_into","","",6,[[["self"]],["result"]]],[11,"get_type_id","","",6,[[["self"]],["typeid"]]],[11,"into_iter","","",7,[[["self"]],["i"]]],[11,"from","","",7,[[["t"]],["t"]]],[11,"into","","",7,[[["self"]],["u"]]],[11,"try_from","","",7,[[["u"]],["result"]]],[11,"borrow","","",7,[[["self"]],["t"]]],[11,"borrow_mut","","",7,[[["self"]],["t"]]],[11,"try_into","","",7,[[["self"]],["result"]]],[11,"get_type_id","","",7,[[["self"]],["typeid"]]],[11,"into_iter","","",8,[[["self"]],["i"]]],[11,"from","","",8,[[["t"]],["t"]]],[11,"into","","",8,[[["self"]],["u"]]],[11,"try_from","","",8,[[["u"]],["result"]]],[11,"borrow","","",8,[[["self"]],["t"]]],[11,"borrow_mut","","",8,[[["self"]],["t"]]],[11,"try_into","","",8,[[["self"]],["result"]]],[11,"get_type_id","","",8,[[["self"]],["typeid"]]],[11,"from","ferrous_chloride::literals","",9,[[["t"]],["t"]]],[11,"into","","",9,[[["self"]],["u"]]],[11,"to_owned","","",9,[[["self"]],["t"]]],[11,"clone_into","","",9,N],[11,"try_from","","",9,[[["u"]],["result"]]],[11,"borrow","","",9,[[["self"]],["t"]]],[11,"borrow_mut","","",9,[[["self"]],["t"]]],[11,"try_into","","",9,[[["self"]],["result"]]],[11,"get_type_id","","",9,[[["self"]],["typeid"]]],[11,"from","","",10,[[["t"]],["t"]]],[11,"into","","",10,[[["self"]],["u"]]],[11,"to_owned","","",10,[[["self"]],["t"]]],[11,"clone_into","","",10,N],[11,"try_from","","",10,[[["u"]],["result"]]],[11,"borrow","","",10,[[["self"]],["t"]]],[11,"borrow_mut","","",10,[[["self"]],["t"]]],[11,"try_into","","",10,[[["self"]],["result"]]],[11,"get_type_id","","",10,[[["self"]],["typeid"]]],[11,"from","ferrous_chloride::value","",11,[[["t"]],["t"]]],[11,"into","","",11,[[["self"]],["u"]]],[11,"to_owned","","",11,[[["self"]],["t"]]],[11,"clone_into","","",11,N],[11,"try_from","","",11,[[["u"]],["result"]]],[11,"borrow","","",11,[[["self"]],["t"]]],[11,"borrow_mut","","",11,[[["self"]],["t"]]],[11,"try_into","","",11,[[["self"]],["result"]]],[11,"get_type_id","","",11,[[["self"]],["typeid"]]],[11,"len_scalar","","",11,[[["self"]],["usize"]]],[11,"len_scalar","ferrous_chloride","",2,[[["self"]],["usize"]]],[11,"is_merged","ferrous_chloride::value","",11,[[["self"]],["bool"]]],[11,"is_unmerged","","",11,[[["self"]],["bool"]]],[11,"is_merged","ferrous_chloride","",1,[[["self"]],["bool"]]],[11,"is_unmerged","","",1,[[["self"]],["bool"]]],[11,"is_merged","","",2,[[["self"]],["bool"]]],[11,"is_unmerged","","",2,[[["self"]],["bool"]]],[11,"next","ferrous_chloride::iter","",3,[[["self"]],["option"]]],[11,"size_hint","","",3,N],[11,"next","","",4,[[["self"]],["option"]]],[11,"size_hint","","",4,N],[11,"next","","",5,[[["self"]],["option"]]],[11,"size_hint","","",5,N],[11,"next","","",6,[[["self"]],["option"]]],[11,"size_hint","","",6,N],[11,"next","","",7,[[["self"]],["option"]]],[11,"size_hint","","",7,N],[11,"next","","",8,[[["self"]],["option"]]],[11,"size_hint","","",8,N],[11,"eq","ferrous_chloride::literals","",9,[[["self"],["key"]],["bool"]]],[11,"ne","","",9,[[["self"],["key"]],["bool"]]],[11,"eq","","",10,[[["self"],["number"]],["bool"]]],[11,"ne","","",10,[[["self"],["number"]],["bool"]]],[11,"eq","ferrous_chloride::value","",11,[[["self"],["value"]],["bool"]]],[11,"ne","","",11,[[["self"],["value"]],["bool"]]],[11,"eq","ferrous_chloride","",1,[[["self"],["oneormany"]],["bool"]]],[11,"ne","","",1,[[["self"],["oneormany"]],["bool"]]],[11,"eq","","",2,[[["self"],["keyvaluepairs"]],["bool"]]],[11,"ne","","",2,[[["self"],["keyvaluepairs"]],["bool"]]],[11,"into_iter","","",1,N],[11,"into_iter","","",2,N],[11,"clone","ferrous_chloride::literals","",9,[[["self"]],["key"]]],[11,"clone","","",10,[[["self"]],["number"]]],[11,"clone","ferrous_chloride::value","",11,[[["self"]],["value"]]],[11,"clone","ferrous_chloride","",1,[[["self"]],["oneormany"]]],[11,"clone","","",2,[[["self"]],["keyvaluepairs"]]],[11,"extend","","",2,[[["self"],["t"]]]],[11,"from","ferrous_chloride::literals","",9,[[["str"]],["self"]]],[11,"from","","",9,[[["string"]],["self"]]],[11,"from","ferrous_chloride::value","",11,[[["number"]],["self"]]],[11,"from","ferrous_chloride::literals","",10,[[["i64"]],["self"]]],[11,"from","","",10,[[["f64"]],["self"]]],[11,"from","ferrous_chloride::value","",11,[[["t"]],["value"]]],[11,"from","","",11,[[["i64"]],["self"]]],[11,"from","","",11,[[["f64"]],["self"]]],[11,"from","","",11,[[["bool"]],["self"]]],[11,"from","","",11,[[["string"]],["self"]]],[11,"from","","",11,[[["vec",["mapvalues"]]],["self"]]],[11,"from","","",11,[[["block"]],["self"]]],[11,"from","","",11,[[["str"]],["self"]]],[11,"from","","",11,[[["option",["vec"]]],["self"]]],[11,"from","","",11,[[["mapvalues"]],["self"]]],[11,"fmt","ferrous_chloride","",0,[[["self"],["formatter"]],["result"]]],[11,"fmt","ferrous_chloride::literals","",9,[[["self"],["formatter"]],["result"]]],[11,"fmt","","",10,[[["self"],["formatter"]],["result"]]],[11,"fmt","ferrous_chloride::value","",11,[[["self"],["formatter"]],["result"]]],[11,"fmt","ferrous_chloride","",1,[[["self"],["formatter"]],["result"]]],[11,"fmt","","",2,[[["self"],["formatter"]],["result"]]],[11,"fmt","","",0,[[["self"],["formatter"]],["result"]]],[11,"hash","ferrous_chloride::literals","",9,[[["self"],["h"]]]],[11,"deref","","",9,N],[11,"index","ferrous_chloride","Warning If the variant is unmerged, this operation will…",2,[[["self"],["q"]],["v"]]],[11,"from_iter","ferrous_chloride::value","",11,[[["t"]],["self"]]],[11,"from_iter","ferrous_chloride","",12,[[["t"]],["self"]]],[11,"from_iter","","",13,[[["t"]],["self"]]],[11,"from_str","ferrous_chloride::literals","",10,[[["str"]],["result"]]],[11,"borrow","","",9,[[["self"]],["str"]]],[11,"name","ferrous_chloride","",0,[[["self"]],["option",["str"]]]],[11,"cause","","",0,[[["self"]],["option",["fail"]]]],[11,"backtrace","","",0,[[["self"]],["option",["backtrace"]]]]],"paths":[[4,"Error"],[4,"OneOrMany"],[4,"KeyValuePairs"],[4,"OneOrManyIterator"],[4,"OneOrManyIntoIterator"],[4,"KeyValuePairsIterator"],[4,"KeyValuePairsIntoIterator"],[4,"KeyIterator"],[4,"ValueIterator"],[4,"Key"],[4,"Number"],[4,"Value"],[6,"Block"],[6,"MapValues"],[8,"ScalarLength"],[8,"Mergeable"]]};
initSearch(searchIndex);addSearchOptions(searchIndex);