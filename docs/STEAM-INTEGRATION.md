# Steam integration

The Rust catalog reads local libraryfolders/appmanifest metadata and bounded
local JPEG artwork. It does not authenticate, download games, implement DRM,
manage Proton or replace Steam Cloud. The official Steam client remains in
control of those operations. Fixed validated actions delegate through Steam
URIs; availability depends on the installed client.

Local preferences are separate from Steam files. Artwork is loaded at runtime
and is not licensed by the project MIT license. Do not redistribute artwork,
account data, personal library inventories or screenshots containing them.
Web demo names are illustrative; the public export includes no demo artwork.
No Steam Web API key or paid service is required by this source snapshot.
