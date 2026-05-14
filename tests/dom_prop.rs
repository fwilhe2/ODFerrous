use odferrous::dom::DocumentModel;
use proptest::prelude::*;

proptest! {
    #[test]
    fn model_roundtrip(heads in prop::collection::vec(".*", 0..5), pars in prop::collection::vec(".*", 0..5)) {
        let model = DocumentModel { headings: heads.clone(), paragraphs: pars.clone() };
        let xml = model.to_content_xml();
        let parsed = DocumentModel::from_content_xml(&xml).expect("parse");
        prop_assert_eq!(model, parsed);
    }
}
