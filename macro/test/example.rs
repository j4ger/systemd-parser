use systemd_parser_macro::UnitConfig;

#[derive(UnitConfig)]
pub struct CustomUnit {

	/// a regular public config field
	pub regular: String,

	/// a private config field
	private: String,

	/// an optional config field
	optional: String,

	/// a vector config field	
	vector: Vec<String>,

	/// a nested config field
	nested: NestedConfig,

	/// a config field with custom key name
	#[key="alt-key"]
	custom_named: String,

	/// a config field with default value
	#[default="default-value"]
	default_valued: String,
}

/// a nested config struct
#[derive(UnitConfig)]
pub struct NestedConfig {
	pub internal: String
}
