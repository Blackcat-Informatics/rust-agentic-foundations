const fs=require('fs');const p='docs/capabilities/source-index.md';let s=fs.readFileSync(p,'utf8');const t='raf_source::test_crate::';
function before(anchor,title,tests){if(!s.includes(anchor))throw Error(anchor);s=s.replace(anchor,'\n\n### '+title+'\n<!-- raf:tested-by '+tests.map(x=>t+x).join(',')+' -->\n\n'+anchor);}
before('Two paths have equal identities','File identity and seen reads',['memory_tree::the_identity_of_a_file_is_its_normalised_key_and_refuses_what_read_refuses','memory_tree::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen','disk_tree_containment::the_identity_of_a_file_is_the_device_and_inode_of_its_verified_handle','disk_tree_containment::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen']);
before('Line ranges are 1-based','Byte and line spans',['symbol_index::items_on_one_long_line_of_wide_characters_get_exact_byte_offsets','source_text::a_one_line_item_has_the_half_open_lines_n_to_n_plus_one']);
before('`Item::parameter_bindings` gives','Parameter bindings',['symbol_index::parameter_bindings_are_every_bound_name_in_signature_order_beside_a_function_with_none']);
before('`Item::initializer` gives','Initializer ranges',['symbol_index::an_initializer_is_the_text_between_the_equals_and_the_semicolon_beside_a_required_constant']);
fs.writeFileSync(p,s);
