// ABOUTME: Accepts one file from the system share sheet into the private local inbox.
// ABOUTME: The main app chooses a session and sends the attachment over its own SSH connection.
import UIKit
import UniformTypeIdentifiers

final class ShareController: UIViewController {
    private let save = UIButton(type: .system)
    private let cancel = UIButton(type: .system)
    private let message = UILabel()
    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .systemBackground
        message.text = "Save this file, then open Compose in a Spaceport terminal to attach it."
        message.numberOfLines = 0
        save.setTitle("Save to Spaceport", for: .normal)
        save.addTarget(self, action: #selector(stage), for: .touchUpInside)
        cancel.setTitle("Close", for: .normal)
        cancel.addAction(UIAction { [weak self] _ in self?.extensionContext?.completeRequest(returningItems: nil) }, for: .touchUpInside)
        let stack = UIStackView(arrangedSubviews: [message, save, cancel])
        stack.axis = .vertical
        stack.spacing = 16
        stack.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(stack)
        NSLayoutConstraint.activate([stack.leadingAnchor.constraint(equalTo: view.layoutMarginsGuide.leadingAnchor),
                                     stack.trailingAnchor.constraint(equalTo: view.layoutMarginsGuide.trailingAnchor),
                                     stack.centerYAnchor.constraint(equalTo: view.centerYAnchor)])
    }

    @objc private func stage() {
        guard let item = extensionContext?.inputItems.first as? NSExtensionItem,
              let provider = item.attachments?.first, let type = provider.registeredTypeIdentifiers.first,
              let directory = SharedDraftInbox.directory else { message.text = "Spaceport could not open this share."; return }
        save.isEnabled = false
        cancel.isEnabled = false
        isModalInPresentation = true
        provider.loadFileRepresentation(forTypeIdentifier: type) { url, error in
            do {
                if let error { throw error }
                guard let url else { throw DraftFile.FileError.unreadable }
                try SharedDraftInbox.save(DraftFile(url: url, suggestedName: provider.suggestedName), in: directory)
                DispatchQueue.main.async { self.extensionContext?.completeRequest(returningItems: nil) }
            } catch {
                DispatchQueue.main.async { self.message.text = error.localizedDescription; self.save.isEnabled = true; self.cancel.isEnabled = true; self.isModalInPresentation = false }
            }
        }
    }
}
