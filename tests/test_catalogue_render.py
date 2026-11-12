from pathlib import Path
import unittest


SOURCE = (Path(__file__).resolve().parents[1] / "src" / "main.rs").read_text()


class CatalogueRenderTests(unittest.TestCase):
    def test_catalogue_uses_its_own_render_target(self):
        self.assertIn("let catalogue_view = make_view(256, 104);", SOURCE)
        self.assertIn("render_shop_preview(\n                &catalogue_view,", SOURCE)
        self.assertNotIn("render_shop_preview(&full_view,", SOURCE)

    def test_preview_is_prepared_before_display_rendering(self):
        frame = SOURCE.index("let dance = if looper.beat || dog_jamming")
        preview = SOURCE.index("render_shop_preview(\n", frame)
        display = SOURCE.index("set_default_camera();\n        clear_background(BLACK);", frame)
        overlay = SOURCE.index("if shop_open {\n            let rows = home::shop_rows();", display)
        self.assertLess(preview, display)
        self.assertLess(display, overlay)
        self.assertNotIn("render_shop_preview(", SOURCE[overlay:])

    def test_catalogue_draws_preview_texture_on_screen(self):
        overlay = SOURCE.index("if shop_open {\n            let rows = home::shop_rows();")
        block = SOURCE[overlay:SOURCE.index("if !picking && !show_help && first_steps", overlay)]
        self.assertIn("draw_texture_ex(\n                    &catalogue_view.texture,", block)
        self.assertIn("flip_y: true,", block)
        self.assertNotIn("set_camera(", block)
        self.assertNotIn("render_shop_preview(", block)

    def test_preview_clears_old_depth_each_frame(self):
        start = SOURCE.index("fn render_shop_preview(")
        end = SOURCE.index("const CAMPFIRE:", start)
        self.assertIn("clear_background(Color::from_rgba(28, 24, 42, 255));", SOURCE[start:end])

    def test_both_players_can_open_and_browse(self):
        self.assertIn("let act = if who == 0 { is_key_pressed(KeyCode::E) } else { is_key_pressed(KeyCode::O) };", SOURCE)
        self.assertIn("let shop_left = if shop_owner == 0", SOURCE)
        self.assertIn("KeyCode::J", SOURCE)
        self.assertIn("KeyCode::L", SOURCE)
        self.assertIn("shop_open = true;\n                        shop_owner = who;", SOURCE)


if __name__ == "__main__":
    unittest.main()
