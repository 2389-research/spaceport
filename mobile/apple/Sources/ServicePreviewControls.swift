// ABOUTME: Opens an explicitly selected loopback HTTP service in the system browser.
// ABOUTME: Keeps the experimental control in debug builds for suspension trials.
#if DEBUG
import SwiftUI

struct ServicePreviewControls: View {
    @ObservedObject var connection: MobileConnection
    @Environment(\.openURL) private var openURL
    @State private var editing = false
    @State private var remote = ""
    @State private var failure: String?
    @State private var busy = false

    var body: some View {
        Group {
            if connection.previewPort != nil {
                Button("Stop service preview", systemImage: "stop.circle") {
                    busy = true
                    Task {
                        do { try await connection.stopServicePreview() } catch { failure = error.localizedDescription }
                        busy = false
                    }
                }
            } else {
                Button("Preview loopback service", systemImage: "safari") { editing = true }
            }
        }
        .disabled(busy)
        .alert("Preview loopback service", isPresented: $editing) {
            TextField("Remote port", text: $remote).keyboardType(.numberPad)
            Button("Cancel", role: .cancel) {}
            Button("Open") {
                guard let port = UInt16(remote), port > 0 else { failure = "Enter a port from 1 to 65535."; return }
                busy = true
                Task {
                    defer { busy = false }
                    do {
                        let local = try await connection.startServicePreview(port)
                        if let url = URL(string: "http://127.0.0.1:\(local)/") {
                            openURL(url) { accepted in
                                if !accepted {
                                    Task {
                                        do { try await connection.stopServicePreview(); failure = "Could not open the browser." }
                                        catch { failure = error.localizedDescription }
                                    }
                                }
                            }
                        }
                    } catch { failure = error.localizedDescription }
                }
            }
        } message: { Text("Open one HTTP port on this SSH host in your browser.") }
        .alert("Service preview", isPresented: $failure.presented) { Button("OK") {} } message: { Text(failure ?? "") }
    }
}
#endif
