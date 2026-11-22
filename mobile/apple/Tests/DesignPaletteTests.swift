// ABOUTME: Checks the app accent as iOS resolves it for light and dark appearances.
// ABOUTME: Uses the compiled app asset catalog so a missing variant fails in the simulator.
import UIKit
import XCTest

final class DesignPaletteTests: XCTestCase {
    func testAccentResolvesInBothAppearances() throws {
        for (style, red, green, blue): (UIUserInterfaceStyle, Int, Int, Int) in [
            (.light, 196, 20, 93), (.dark, 255, 92, 154)
        ] {
            let traits = UITraitCollection(userInterfaceStyle: style)
            let accent = try XCTUnwrap(UIColor(named: "AccentColor", in: .main, compatibleWith: traits))
            var r: CGFloat = 0, g: CGFloat = 0, b: CGFloat = 0, a: CGFloat = 0
            XCTAssertTrue(accent.resolvedColor(with: traits).getRed(&r, green: &g, blue: &b, alpha: &a))
            XCTAssertEqual(r * 255, CGFloat(red), accuracy: 0.5)
            XCTAssertEqual(g * 255, CGFloat(green), accuracy: 0.5)
            XCTAssertEqual(b * 255, CGFloat(blue), accuracy: 0.5)
            XCTAssertEqual(a, 1)
        }
    }
}
