---
layout: default
title: Add Devices
nav_order: 2
parent: User Guides
description: >
  To add devices to Firezone, follow these commands.
---
---

**We recommend asking users to generate their own device configs so the private
key is exposed only to them.** Users can follow instructions on the
[Client Instructions]({%link docs/user-guides/client-instructions.md%})
page to generate their own device configs.

## Admin device config generation

Firezone admins can generate device configs for all users. This can be done by
clicking the "Add Device" button on the user profile page found in `/users`.

![add device under user](https://user-images.githubusercontent.com/52545545/153467794-a9912bf0-2a13-4d05-9df9-2bd6e32b594c.png){:width="600"}

Once the device profile is created, you can send the WireGuard configuration
file to the user.

Devices are associated with users. See [Add Users
]({%link docs/user-guides/add-users.md%}) for more information on how to add
a user.

\
[Related: Client Instructions]({%link docs/user-guides/client-instructions.md%}){:.btn.btn-purple}
