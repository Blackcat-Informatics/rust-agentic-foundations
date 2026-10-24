---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-normal-form-verification
subject: cap:census-normal-form#verification
inputs:
  - kind: section
    name: cap:census-normal-form#verification
    digest: 2a28a22a13ba9ebc8d9e117bb576ec843091f819bcd623badc3c100803b52649
  - kind: symbol
    name: raf_census::Baseline
    digest: 79033c79f4d11654eaca8e0765fcd3ae438248382961ae0bb77f0bebfcc5b66b
  - kind: symbol
    name: raf_census::Baseline::from_ntriples
    digest: 3d3d3fb2dfab3d7633af1135ed8520921e9ca26b02095d00e6267964ae215b69
  - kind: symbol
    name: raf_census::Baseline::from_turtle
    digest: a7adbe3825924fb52e0439dd1b08bd4d45f9698008e1d83c51a692c4d35889b0
  - kind: symbol
    name: raf_census::Baseline::to_ntriples
    digest: 1920b9c5ca3c3355766a338a27dce1589427cbe5da7d42b9c288327b75b77544
  - kind: symbol
    name: raf_census::Baseline::to_turtle
    digest: 17e7456e524ca6a934ffbd7156dbadc6e88852606713c52b31f43fb136bf0f8b
  - kind: symbol
    name: raf_census::BaselineGroup
    digest: 9486ec179ca770186234c26ecc6f51c25cc26334bd571281ffe89d138a5904e1
  - kind: symbol
    name: raf_census::Census
    digest: 19870102c9dc156d8562185b2814d89bac18bff9fae2a5dc682b97e62a8e2f47
  - kind: symbol
    name: raf_census::Census::baseline
    digest: 96e7a3a451b71f0d553632dae4fc5292ef260c01b380bf3c003e3ba910647b53
  - kind: symbol
    name: raf_census::Census::body_groups
    digest: 527fdf3ff8cdea454013daf1816b8e8307bd7e2bb7dbd938aabf4f18defe0f8d
  - kind: symbol
    name: raf_census::Census::file
    digest: e6a5a4a6bbb9e89476995b139e85605d386ec19376c092683e85114d7c973c9d
  - kind: symbol
    name: raf_census::Census::files
    digest: 42b7861814db5fcab55ff74dac08d8baae1d831cc5c7c31e332adf24e97ecd3b
  - kind: symbol
    name: raf_census::Census::groups
    digest: 80d4052fa23b59709ee188531b2eaa34db43da26936a53ceebc0465c9393af1a
  - kind: symbol
    name: raf_census::Census::hex_tables
    digest: f859dd2ff349b68907b45292f05f87bf1c8485c2d433ec8e16807a166caa8c85
  - kind: symbol
    name: raf_census::Census::index
    digest: bb132b08e167fc9fd624e775777ed8e6ec84f789ce8076fb914f31089ce39259
  - kind: symbol
    name: raf_census::Census::load
    digest: 96ff33527e1e19f9d2c99e340d236918e767ddcf556bed725f10495a4970145b
  - kind: symbol
    name: raf_census::Census::match_jobs
    digest: 148ce4fe5b8e66d5ff60705612b691c9782b9359c0e64b33d4fd2baa78c8c100
  - kind: symbol
    name: raf_census::Census::options
    digest: 2b20fda60a5cf765894630605403652db730ae52e1dcaa16fb22e4dad6f60a11
  - kind: symbol
    name: raf_census::Census::shared_constants
    digest: 02ead0cfd9e2945789421fb4730343238ede2c97d20a3c020607c594745bcadb
  - kind: symbol
    name: raf_census::Census::shim_groups
    digest: 255985d84a9159b3175c00ecdad4b92d4d908710d41fdd4e138c38bc8eca7700
  - kind: symbol
    name: raf_census::Census::unit
    digest: d7e82d4dc4bc631f5150db9c55a4dbce3c966d3ce2052f50a2e7c31bf7cff277
  - kind: symbol
    name: raf_census::Census::units
    digest: 675d80fbc16cdd1d51784a395151f41dfa5172ab8d383588f4b07a45d12a0e05
  - kind: symbol
    name: raf_census::Census::units_of
    digest: c804a5d715f3800597674844642fe7789f1d1f51ed5e5bb8f46d6237c14844ac
  - kind: symbol
    name: raf_census::CensusOptions
    digest: 3374a4ce9c59f9ee5a5444a70a1803f5a560846fb99259baa6bad8ebf721ed2c
  - kind: symbol
    name: raf_census::CensusOptions::max_file_bytes
    digest: f2b8608bc6fa2a63b793880707e3d72480ef2702447f4d0a317c85d090ed0922
  - kind: symbol
    name: raf_census::CensusOptions::min_tokens
    digest: 8df57f2ea55e951529af334bdad02dd05b2f8cf08425583f147a467ff6a9d573
  - kind: symbol
    name: raf_census::ConstantRow
    digest: 2ce9c518db4798b21bb51b6da5cd4f23934a968c98d826454ebe4a6166676abe
  - kind: symbol
    name: raf_census::DEFAULT_MAX_FILE_BYTES
    digest: b6b005898d82e958376a4867c9199b5d5fb3a20566d0c57db50368b3c42e5ec1
  - kind: symbol
    name: raf_census::DEFAULT_MIN_TOKENS
    digest: a6b9402e3b2712275ff714c0ffd35b5ca936c62dcabeb63b1b12c36d1e2b682f
  - kind: symbol
    name: raf_census::Divergence
    digest: 1041f47b15ea7f2f6e7d16ef3143f98fdcc61d6b27cb75d64ac74021cb6228ea
  - kind: symbol
    name: raf_census::ENCODING
    digest: ed26a671fea5e33a046208e05e5c69d72cf3b995dc2c615d9188ce290b332211
  - kind: symbol
    name: raf_census::Error
    digest: ab94c74b8606fbcf2b3ac261718d7f02c3c8ac8e89ae1afee016347d334c6438
  - kind: symbol
    name: raf_census::FileCtx
    digest: dc820cd4bb1dfd693cf6cb0f66493c8438837ee5177cc381ba2cc9a94be75afe
  - kind: symbol
    name: raf_census::Fingerprint
    digest: adc45c51c55b4610c80dbe7d48fd3704c26ccdee1e6ceea7cb9ba65d3ddbb821
  - kind: symbol
    name: raf_census::Fingerprint::abbreviated
    digest: dbe76be2f4bf60a8361648984ccf7710f3d4791d2b03a3f6adb6c1b73acaeb54
  - kind: symbol
    name: raf_census::Fingerprint::body
    digest: 2b854a575414e8b5dda1067acda06f67e8e3f396838793dd6d0df98cb34da8c3
  - kind: symbol
    name: raf_census::Fingerprint::kind
    digest: eb4b48fc22497b1f5c7cc18395487480ba1b8c49b9d4b6ce9aca52ee266612d5
  - kind: symbol
    name: raf_census::Fingerprint::shim
    digest: eb15e75c3c1c314523e4b657523065ecc897f1d25e1002dacc0b94908be1ad78
  - kind: symbol
    name: raf_census::Form
    digest: 46cebbabba2086ba8444ce47932338a69fc870ddad13647a5e6add72afa2b5e3
  - kind: symbol
    name: raf_census::Form::is_empty
    digest: 3c88fa93b7d8c2e9298714111ffc67ead2a154d6db0b1e2de7a0e003a2c9cb29
  - kind: symbol
    name: raf_census::Form::len
    digest: 1be76935d4254c218510323b79bfb4c0fc2902069d238eb74ff5c6dd7e9b1cf4
  - kind: symbol
    name: raf_census::Form::texts
    digest: dc95fa94e2d94f4ea5b2eafe930a76c51ab9d1fd50ba5c932cec647942e94fe3
  - kind: symbol
    name: raf_census::Form::tokens
    digest: b5bbce341eab299f132c8b5f16a9c374850834f0e6718da1e0043d8b9113c05d
  - kind: symbol
    name: raf_census::Group
    digest: f8fcff2f4d016da77da83caa8572db7a837ee3de72ee5dfa6afca4533d16dc95
  - kind: symbol
    name: raf_census::Group::member_symbols
    digest: 68357a7ba891be71383bfeb4e648d7443bbb1362505a98d15d12cfb2c29c22c2
  - kind: symbol
    name: raf_census::GroupKind
    digest: 4ad8dfee4a8a6c69498e55f4f2b32044e3b215e14528aab9655fa8bd221ac182
  - kind: symbol
    name: raf_census::HexTable
    digest: e6fcb7c28ae863ded739cae83eaa7c147891bb7b1b38d546652e7684f50f43b8
  - kind: symbol
    name: raf_census::Index
    digest: c67be4134bbae18ede6b78dcb1d3c0c020b7e93f883b316020daf6717b44d834
  - kind: symbol
    name: raf_census::Index::from_ntriples
    digest: 6de0c429c5cd414a37a015a5cc93673fdc7b52b7eec328f11f1a4b5155f22567
  - kind: symbol
    name: raf_census::Index::from_turtle
    digest: d26fb68d344ad127097329df7fcaeaceb27896175587079918485e9952b49de3
  - kind: symbol
    name: raf_census::Index::to_ntriples
    digest: 125bb5af1f05b3a5e79788d13c636f9420d03a7dc1dd31f6676704b6da5bbd4e
  - kind: symbol
    name: raf_census::Index::to_turtle
    digest: 53c301e2e2b26d31cfe2038fa5773005b3d2a0a55ae440af804acbcdf2756da3
  - kind: symbol
    name: raf_census::IndexGroup
    digest: 526186720fbb8de5f01a41d327a799da1535f901ce5d0238e4c2305f9c6c6f61
  - kind: symbol
    name: raf_census::IndexUnit
    digest: 7caf0c81bd8f4025ecba87f64045e6a42364a46e07055c52c05d56f4b627b84a
  - kind: symbol
    name: raf_census::Job
    digest: 3f9d93038d32676c236ad951441859c4141dbbc4a7ae6315e6fcdb6de9611496
  - kind: symbol
    name: raf_census::Job::entry_points
    digest: 4e3d79a1765887484f5fbfeb63882ae6a2167abf124699476ad402a8f832b7a9
  - kind: symbol
    name: raf_census::Job::forbidding
    digest: 9338796c51815c6c86befaba3a32cf10dc349d2f1e532189739ec0b2787b97ec
  - kind: symbol
    name: raf_census::Job::new
    digest: b67f9ffff0720b2c19566b095ab4b7cac1b92e6b0d17cbf7c8d9e2a7ad0bfafc
  - kind: symbol
    name: raf_census::Job::variants
    digest: 2a94d4481bcdbeb4bdb121a86fcf0bd62207a6e12a2cc89a3f14e5cc3ebc2484
  - kind: symbol
    name: raf_census::JobForbidden
    digest: 867f248bb0be451cc449e9f20d34ab159228a8516798ac4f8ed893ee7c107015
  - kind: symbol
    name: raf_census::JobMatch
    digest: 708dbaabe78efaf4a5e9c7c0bd24b75374673e1ad3b188fe99c2f320fe51e3a3
  - kind: symbol
    name: raf_census::JobReport
    digest: e302ee1a65face3346fc7117755ebbec2b9d053aba29cc87fcc2729a06f6cb50
  - kind: symbol
    name: raf_census::JobRow
    digest: 482bddb2df658043edd07d2d6440411f2df1132d34cdb0fb0e88f4b3895bda89
  - kind: symbol
    name: raf_census::LITERAL_TIER
    digest: acd5bf63e3cd092eb29f666d1627584ea79bdd0729f192ea934598d95f5f3aa1
  - kind: symbol
    name: raf_census::Level
    digest: 9ffe45f14aa25208bc073af3ee068ed210a57082ee1fc67028f421e2cce15ca6
  - kind: symbol
    name: raf_census::Level::ALL
    digest: 97dbc46938f70459297a729b4c01e034f23de87de8bd684ba0374000cda05732
  - kind: symbol
    name: raf_census::Level::id
    digest: 53073b5cc885eaf304f5eba6d7eea20c58e9b3a6433e9021917b25f5c54c9880
  - kind: symbol
    name: raf_census::MAX_NESTING
    digest: a4160a54b236b84066635ab11d2ee22cfc91166c265bd5dd72d664d1684d95b1
  - kind: symbol
    name: raf_census::MIN_SHIM_TOKENS
    digest: 54f2b932b7b6d424762c9c8f5ad66367feb972884de3b2cc403a3b5b90ecee71
  - kind: symbol
    name: raf_census::MatchClass
    digest: 23cf505d55c3740521a64158f01fd48dd265a0c9b2e0899b300e4367bf222188
  - kind: symbol
    name: raf_census::MatchRow
    digest: 2c516882f49bc23aac9e818ae8f92e0ce269f7f5a77c91d2b8983e1b1ef6552b
  - kind: symbol
    name: raf_census::Member
    digest: 77794ad5ebd349c424355c15aa52ca35b49d16985ca9d93117a43f0c45c6aace
  - kind: symbol
    name: raf_census::MemberRow
    digest: 2e18418f61caf8e0c55af19aa50c2a7bbfdfcac0ad7869b215980f4bf85fde33
  - kind: symbol
    name: raf_census::NORMAL_FORM
    digest: 70e3fd42fe1cc7ca511c05dbe4bba480e6de8865632b4e51b4a6efb61b02f802
  - kind: symbol
    name: raf_census::Normalizer
    digest: cfee6aa9bbccb8d656a8fe885da6ff8fd38aa68c89f19d713dc7ebc194c6e773
  - kind: symbol
    name: raf_census::Normalizer::explain
    digest: 83ed59bbc24715c894fdc18cb99720df8d9118b7efdae49cc10fc3f389492873
  - kind: symbol
    name: raf_census::Normalizer::new
    digest: 70bcbcbe9762a8cd8634aeb852be3ffdd09c739579f00822a1566bd32628055f
  - kind: symbol
    name: raf_census::Normalizer::print
    digest: 05700ad76b7367561795b67aec849859873735b81cd43258b8ce44516f168406
  - kind: symbol
    name: raf_census::Normalizer::tower
    digest: 6caa955c14562089877f373058ee9d6ed0ed25af7d0d122f80b254bc81dfb09c
  - kind: symbol
    name: raf_census::PASSES
    digest: ba1aa80e87843de29f724ae62f9f27643070937639461ba9615b8cfc49f9cb26
  - kind: symbol
    name: raf_census::ParseFingerprintError
    digest: 036eadc8ec0215d5c97bec57450801489c9880f9857685ef57cc45ed7286b175
  - kind: symbol
    name: raf_census::Print
    digest: d8980657b9a90b4db0436bc4a0b9f874eb19915aa29f15faadf6175342c0ec8c
  - kind: symbol
    name: raf_census::REPORT_FORM
    digest: 43794c2ec681dcd8489f1ab467ce40b094524d2ec54af48f201a916baf361bcf
  - kind: symbol
    name: raf_census::ReportError
    digest: 7e097c78607fccf5d08cd799b9acd6fbe193b667521eb8a6504f126d8b6ca475
  - kind: symbol
    name: raf_census::RuleId
    digest: 34c0fc8f56b2eab01a0852a91c6cb8ff7871c60fca0a75ee6f02c608905d87b3
  - kind: symbol
    name: raf_census::RuleId::as_str
    digest: 5a3a2181673435534898b2e73556d9576f0a1319ecc5dae1dc2ac6eba3718335
  - kind: symbol
    name: raf_census::RuleId::new
    digest: 4384d865f27c93bf42a71ba6360a68fba78a2af9486f52a83cacb01cd7240063
  - kind: symbol
    name: raf_census::SHIM_MAX_TOKENS
    digest: b799e30dd83cb15dc8196c1cea5c1d1cf90a18cef86e69d1e249b750266a6e5e
  - kind: symbol
    name: raf_census::SharedConstant
    digest: 1e08947e80c405107961b08e04ce3d387714aae837495ba8831a9089e504d6ef
  - kind: symbol
    name: raf_census::Slice
    digest: 0e315cbe3fce83044066d1b194b30aac0f55b9c9e29c88a99c87e215ab91017f
  - kind: symbol
    name: raf_census::Slice::new
    digest: 1ed3b50b4175669363644704b67174625d44978b26303de68e8b34aab949b587
  - kind: symbol
    name: raf_census::Slice::params
    digest: 5793d594036228874856b1210d7ce7879b41655a1f8cd77b734d80e64902a7e9
  - kind: symbol
    name: raf_census::StringKind
    digest: 34d33f398618bb97567e9e82cc20f03cf9407ee8837a0b1f6d24633b7316fef2
  - kind: symbol
    name: raf_census::StringLiteral
    digest: 1eeee33ada4229745f19bd1f040f000420bedb94e43f6282beb12b4d40793ddf
  - kind: symbol
    name: raf_census::TableKind
    digest: 25cd08a0f917396e65f7f2b541a09a6663f65ae942618e742cc052e06f1d377b
  - kind: symbol
    name: raf_census::TableKind::name
    digest: e23ab966f3da93acd1628ba8b346fc5cf39a7f5b13c76ab1c1276f00cf6fad58
  - kind: symbol
    name: raf_census::TableRow
    digest: c10b62545c9722e88b03c766b5674652c3d66b5a74936c2c02708d5028e99bae
  - kind: symbol
    name: raf_census::Token
    digest: 346b9c23f15e4a522576693949ca03e162735122859ae6ffa49bf702dc3c713d
  - kind: symbol
    name: raf_census::TokenDiff
    digest: 7c3bc4f36d792205e84d7e401e76f198844471a3e65e9c4d8557935c73c9796c
  - kind: symbol
    name: raf_census::Tower
    digest: 8ec2bfc7089fba46f272c24c0bd525fe65ce8019f0abca21f72dd5875314cea5
  - kind: symbol
    name: raf_census::Tower::calls_through_self
    digest: 2c9dd7f30ebc1f14f6c44584c43f8dd01a4212b5fd9c03e2296f05373cea3775
  - kind: symbol
    name: raf_census::Tower::form
    digest: c6b42a8431c2e335eaeeeaac33c8ec57b5d52bdb898db6c2d2fb0d1342901548
  - kind: symbol
    name: raf_census::Tower::literal_values
    digest: 57c46dbf4fbb532ecce55ec553c05e0acfbaf8872d8e42286c84d39b7375d8e8
  - kind: symbol
    name: raf_census::Tower::new
    digest: 5939e3bd07fd26cafc5aec310706bc4f30c3ab420491cb38335da8ff8513ee4f
  - kind: symbol
    name: raf_census::Tower::print
    digest: 68f136aa330c46f88d0f9080f712bef887e66eead66f59f5b073777a394b2126
  - kind: symbol
    name: raf_census::Tower::shim_candidate
    digest: def8a1b3c38dd5cea2a14a653c4d94343790a0269d36ccd2c6529266c517d6ea
  - kind: symbol
    name: raf_census::Tower::value_tokens
    digest: 5663162682fe9f6e586110d75d2b732184696beb853ae21c8208a861ea084b2d
  - kind: symbol
    name: raf_census::Unit
    digest: 40c99f27e139d7be0b7bc135e539a6c4b5423a9d220e091c3530fc6ee8768a77
  - kind: symbol
    name: raf_census::UnitKind
    digest: 442d0de85e7a025ffc99b02e3af0483042db1a33d9eba0e9e73b612d8d5702d2
  - kind: symbol
    name: raf_census::VALUE_ADAPTERS
    digest: 9100a960c0cf4235ad4942ca485b675927ad21a4643cb69e08f7e7f91baf1391
  - kind: symbol
    name: raf_census::WEAK_KEYWORDS
    digest: '0bc043da387671195ecfef4ce0d510e3c6336a70e2e814affe82a1e8e032fd08'
  - kind: symbol
    name: raf_census::check_normal_form
    digest: 06ce3b7c873c268047a884b76f018848a7bb7fcd51f95b8c5b41c55b596a3812
  - kind: symbol
    name: raf_census::manifest
    digest: 024665fb57e1d0347781572f35b5b8c7bdc9d60cf9080a46f41d5551ec771327
  - kind: symbol
    name: raf_census::report
    digest: ea7ffb2e8bb6c2dcdcce911ce38ac6805d6d8d1875a81712bf7e42646b55df0e
  - kind: symbol
    name: raf_census::report::vocab
    digest: a9bd9d5dcc26ae8015f7099f1a676672669aeeb6a0e876bfe1b518740aac67f1
  - kind: symbol
    name: raf_source_fuzz::bin_crate::census_normalize::exercise
    digest: 0934837fdcc9f4735d5fb6253b367cd0cdee1489a4ba58a85d0e083d2f4f4941
  - kind: symbol
    name: raf_source_fuzz::bin_crate::census_report_read::exercise
    digest: 22da690f45078fc78110ea29b100df5c50838c519b894c385322d7ebbcad2bfc
verdict: accepted
by: agent
---
