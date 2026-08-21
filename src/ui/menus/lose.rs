pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        EguiPrimaryContextPass,
        retry_menu.run_if(OnEnter(Menu::Lose)),
    );
}
fn retry_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Lose);
}
