fn main() -> iced::Result {
    localization::init();
    common::run::<localization::Page>()
}
