// ABOUTME: Checks real profile-ID parsing without enabling a keyboard or sending input.
// ABOUTME: Rejects malformed and mismatched languages before any native fixture can mutate state.
using System;

public static class SpaceportProfileIdTests {
  static void Equal(string expected, string actual) {
    if (expected != actual) throw new InvalidOperationException("Profile ID did not retain its exact KLID");
  }
  static void Refused(ushort language, string value) {
    try { SpaceportProfileId.Klid(language, value); }
    catch (InvalidOperationException) { return; }
    throw new InvalidOperationException("Malformed or mismatched profile ID was accepted");
  }
  public static void Run() {
    Equal("00000409", SpaceportProfileId.Klid(0x0409, "0x0409:0x00000409"));
    Equal("00000415", SpaceportProfileId.Klid(0x0415, "0415:00000415"));
    Equal("A0000409", SpaceportProfileId.Klid(0x0409, "0X0409:0Xa0000409"));
    Equal("00000409", SpaceportProfileId.Klid(0x0409, "00000409"));
    Equal("00000415", SpaceportProfileId.Klid(0x0415, "0x00000415"));
    Refused(0x0409, null);
    Refused(0x0409, "00000409:00000409");
    Refused(0x0409, "0415:00000415");
    Refused(0x0409, "0409:00000415:00000409");
    Refused(0x0409, "409:00000409");
    Refused(0x0409, "0409:0000409");
    Refused(0x0409, "0409:0000040Z");
    Refused(0x0409, "0409:00000409 ");
    Refused(0x0409, " 0409:00000409");
    Refused(0x0409, "0409:00000409\n");
  }
  public static int Main() {
    try { Run(); Console.WriteLine("PROFILE_ID tests PASS"); return 0; }
    catch (Exception error) { Console.Error.WriteLine(error.Message); return 1; }
  }
}
