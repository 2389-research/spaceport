// ABOUTME: Parses enabled Windows keyboard profile identities before native fixture mutation.
// ABOUTME: Keeps profile recognition pure so its rejection rules run without Windows APIs.
using System;
using System.Globalization;

public static class SpaceportProfileId {
  public static string Klid(ushort language, string id) {
    if (id == null || id.Length > 17)
      throw new InvalidOperationException("Enabled keyboard profile has an unsupported identity length");
    string[] fields = id.Split(':');
    if (fields.Length == 1) return Hex(fields[0], 8);
    if (fields.Length != 2)
      throw new InvalidOperationException("Enabled keyboard profile has too many identity fields");
    string declaredLanguage = Hex(fields[0], 4);
    string klid = Hex(fields[1], 8);
    if (UInt16.Parse(declaredLanguage, NumberStyles.AllowHexSpecifier, CultureInfo.InvariantCulture) != language)
      throw new InvalidOperationException("Enabled keyboard profile language does not match its identity");
    return klid;
  }
  static string Hex(string value, int digits) {
    if (value.StartsWith("0x", StringComparison.OrdinalIgnoreCase)) value = value.Substring(2);
    if (value.Length != digits) throw new InvalidOperationException("Enabled keyboard profile has an invalid field width");
    foreach (char character in value)
      if (!Uri.IsHexDigit(character)) throw new InvalidOperationException("Enabled keyboard profile has an invalid hexadecimal field");
    return value.ToUpperInvariant();
  }
}
