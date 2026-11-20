//! Backend-only record definitions grouped by the suite that exercises them.
use super::record::{Record, RecordField, RecordMethod};
const EMPTY: Record = Record {
    name: "",
    generics: "",
    interfaces: "",
    fields: &[],
    methods: &[],
    drops: &[],
    constants: &[],
    copy: false,
    private: false,
};
// Static fixture slices are promoted in the match arms below.
macro_rules! field {
    ($name:literal, $ty:literal, $private:literal) => {
        RecordField {
            name: $name,
            ty: $ty,
            private: $private,
        }
    };
}
macro_rules! method {
    ($source:literal, $private:literal) => {
        RecordMethod {
            source: $source,
            private: $private,
        }
    };
}
mod borrowed_destructors;
mod borrowed_returns;
mod borrowed_storage;
mod classes;
mod entries;
mod function_values;
mod generics;
mod inheritance;
mod interfaces;
mod methods;
mod mir;
mod mir_verify;
mod modules;
mod native;
mod ownership;
mod patterns;
mod privacy;
mod raw_pointers;
mod raw_storage;
mod record_layout;
mod scalar_buffer;
mod shared;
mod unsafe_operations;
pub(super) fn get(id: &str) -> Option<Record> {
    match id {
        "record_0" | "record_1" | "record_2" | "record_3" | "record_17" => native::get(id),
        "record_4" | "record_35" | "record_36" | "record_37" | "record_38" | "record_39" => {
            borrowed_destructors::get(id)
        }
        "record_5" | "record_15" | "record_26" | "record_27" | "record_28" | "record_29"
        | "record_30" | "record_31" | "record_32" => borrowed_returns::get(id),
        "record_6" => entries::get(id),
        "record_7" | "record_8" | "record_9" => function_values::get(id),
        "record_10" | "record_11" | "record_12" | "record_13" | "record_14" | "record_16"
        | "record_33" | "record_34" => methods::get(id),
        "record_18" | "record_19" | "record_20" | "record_21" | "record_22" | "record_23"
        | "record_24" => record_layout::get(id),
        "record_25" => patterns::get(id),
        "record_40" | "record_41" | "record_133" | "record_134" | "record_135" | "record_136"
        | "record_137" | "record_138" | "record_139" | "record_140" | "record_141"
        | "record_142" | "record_143" | "record_144" | "record_145" | "record_146"
        | "record_147" | "record_148" | "record_149" | "record_150" | "record_151"
        | "record_152" | "record_153" | "record_154" | "record_155" | "record_156"
        | "record_157" | "record_158" => shared::get(id),
        "record_42" | "record_43" | "record_44" | "record_45" | "record_46" | "record_47"
        | "record_48" | "record_49" | "record_50" | "record_51" | "record_52" | "record_53"
        | "record_54" | "record_55" | "record_56" | "record_57" | "record_58" | "record_59"
        | "record_60" | "record_61" | "record_62" | "record_63" | "record_64" | "record_65"
        | "record_66" | "record_67" | "record_68" | "record_69" | "record_70" | "record_71"
        | "record_72" | "record_73" | "record_74" | "record_75" | "record_76" | "record_77"
        | "record_78" | "record_79" => interfaces::get(id),
        "record_80" => ownership::get(id),
        "record_81" | "record_83" | "record_84" | "record_85" | "record_86" | "record_87"
        | "record_88" | "record_89" | "record_90" | "record_91" => raw_storage::get(id),
        "record_82" | "record_92" | "record_93" => raw_pointers::get(id),
        "record_94" => generics::get(id),
        "record_95" => inheritance::get(id),
        "record_96" | "record_97" | "record_98" | "record_99" | "record_100" | "record_101"
        | "record_102" | "record_103" | "record_104" | "record_105" | "record_106"
        | "record_107" | "record_108" => borrowed_storage::get(id),
        "record_109" | "record_110" | "record_111" | "record_112" | "record_113" | "record_114"
        | "record_115" | "record_116" | "record_117" => privacy::get(id),
        "record_118" => classes::get(id),
        "record_119" => mir_verify::get(id),
        "record_120" | "record_121" | "record_122" | "record_123" => modules::get(id),
        "record_124" => mir::get(id),
        "record_125" | "record_126" | "record_127" | "record_128" | "record_129" | "record_130"
        | "record_131" | "record_132" => unsafe_operations::get(id),
        "record_159" => scalar_buffer::get(id),
        _ => None,
    }
}
