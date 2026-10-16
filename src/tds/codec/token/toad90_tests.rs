    use super::*;
    use crate::tds::codec::VarLenContext;

    fn column(name: &str, ty: TypeInfo) -> MetaDataColumn<'static> {
        MetaDataColumn {
            base: BaseMetaDataColumn {
                flags: BitFlags::empty(),
                ty,
                table_name: None,
            },
            col_name: Cow::Owned(name.to_string()),
        }
    }

    fn sized(ty: VarLenType, len: usize) -> TypeInfo {
        TypeInfo::VarLenSized(VarLenContext::new(ty, len, None))
    }

    #[test]
    fn the_name_is_bracket_quoted() {
        assert_eq!(
            column("Order Date", TypeInfo::FixedLen(FixedLenType::Int4)).to_string(),
            "[Order Date] int"
        );
        assert_eq!(
            column("a]b", TypeInfo::FixedLen(FixedLenType::Int8)).to_string(),
            "[a]]b] bigint"
        );
    }

    #[test]
    fn unicode_columns_are_spelled_in_characters_and_max_only_for_the_marker() {
        assert_eq!(column("s", sized(VarLenType::NVarchar, 600)).to_string(), "[s] nvarchar(300)");
        assert_eq!(column("s", sized(VarLenType::NVarchar, 8000)).to_string(), "[s] nvarchar(4000)");
        assert_eq!(column("s", sized(VarLenType::NVarchar, 0xFFFF)).to_string(), "[s] nvarchar(max)");
        assert_eq!(column("s", sized(VarLenType::NChar, 10)).to_string(), "[s] nchar(5)");
        // Byte-sized types keep their byte length.
        assert_eq!(column("s", sized(VarLenType::BigVarChar, 8000)).to_string(), "[s] varchar(8000)");
        assert_eq!(column("s", sized(VarLenType::BigVarChar, 0xFFFF)).to_string(), "[s] varchar(max)");
        assert_eq!(column("s", sized(VarLenType::BigChar, 5)).to_string(), "[s] char(5)");
        assert_eq!(column("b", sized(VarLenType::BigVarBin, 100)).to_string(), "[b] varbinary(100)");
        assert_eq!(column("b", sized(VarLenType::BigVarBin, 0xFFFF)).to_string(), "[b] varbinary(max)");
        assert_eq!(column("b", sized(VarLenType::BigBinary, 4)).to_string(), "[b] binary(4)");
    }

    #[cfg(feature = "tds73")]
    #[test]
    fn temporal_columns_carry_their_scale() {
        assert_eq!(column("t", sized(VarLenType::Timen, 0)).to_string(), "[t] time(0)");
        assert_eq!(column("t", sized(VarLenType::Timen, 3)).to_string(), "[t] time(3)");
        assert_eq!(column("t", sized(VarLenType::Datetime2, 7)).to_string(), "[t] datetime2(7)");
        assert_eq!(
            column("t", sized(VarLenType::DatetimeOffsetn, 3)).to_string(),
            "[t] datetimeoffset(3)"
        );
        assert_eq!(column("d", sized(VarLenType::Daten, 3)).to_string(), "[d] date");
    }

    #[test]
    fn nullable_datetime_and_money_are_told_apart_by_their_length() {
        assert_eq!(column("d", sized(VarLenType::Datetimen, 4)).to_string(), "[d] smalldatetime");
        assert_eq!(column("d", sized(VarLenType::Datetimen, 8)).to_string(), "[d] datetime");
        assert_eq!(column("m", sized(VarLenType::Money, 4)).to_string(), "[m] smallmoney");
        assert_eq!(column("m", sized(VarLenType::Money, 8)).to_string(), "[m] money");
    }
