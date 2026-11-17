// ABOUTME: Points the download buttons at the newest preview disk image on GitHub Releases.
// ABOUTME: Also starts the typing animation when the illustrated window scrolls into view.

const RELEASES_API = "https://api.github.com/repos/2389-research/spaceport/releases?per_page=10";

// Previews are prereleases, so GitHub's "latest release" link does not find
// them. Without this lookup the buttons keep their plain link to the list.
async function pointAtNewestPreview() {
  const response = await fetch(RELEASES_API, { headers: { Accept: "application/vnd.github+json" } });
  if (!response.ok) return;
  const releases = await response.json();
  for (const release of releases) {
    if (release.draft) continue;
    const image = release.assets.find((asset) => asset.name.endsWith(".dmg"));
    if (!image) continue;
    for (const link of document.querySelectorAll("[data-download]")) {
      link.href = image.browser_download_url;
    }
    const version = release.tag_name.replace(/^v/, "");
    for (const label of document.querySelectorAll("[data-version]")) {
      label.textContent = `Preview ${version}`;
    }
    return;
  }
}

pointAtNewestPreview().catch(() => {});

const plate = document.querySelector(".plate");
if (plate && "IntersectionObserver" in window) {
  const observer = new IntersectionObserver(
    (entries) => {
      if (!entries.some((entry) => entry.isIntersecting)) return;
      plate.classList.add("play");
      observer.disconnect();
    },
    { threshold: 0.5 },
  );
  observer.observe(plate);
}
