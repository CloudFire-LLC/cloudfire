---
layout: default
title: Okta
nav_order: 2
parent: Authenticate
description: >
  This page contains instructions on setting up Okta
  as the SSO provider for Firezone.
---
---

Firezone supports Single Sign-On (SSO) through Okta.
After successfully configuring SSO with Firezone, users will be prompted to sign
in with their Okta credentials in the Firezone portal to authenticate VPN
sessions and download device configuration files.

![Firezone Okta SSO Login](https://user-images.githubusercontent.com/52545545/156855886-5a4a0da7-065c-4ec1-af33-583dff4dbb72.gif){:width="600"}

To set up SSO, follow the steps below:

## Step 1 - Create Okta App Integration

_This section of the guide is based on
[Okta's documentation](https://help.okta.com/en/prod/Content/Topics/Apps/Apps_App_Integration_Wizard_OIDC.htm)._

In the Admin Console, go to `Applications > Applications` and click `Create App Integration`.
Set `Sign-in method` to `OICD - OpenID Connect` and `Application type` to `Web application`.

![Okta Create App Integration](https://user-images.githubusercontent.com/52545545/155907051-64a74d0b-bdcd-4a22-bfca-542dacc8ad20.png){:width="800"}

![Okta Create Options](https://user-images.githubusercontent.com/52545545/155909125-25d6ddd4-7d0b-4be4-8fbc-dc673bb1f61f.png){:width="800"}

On the following screen, configure the following settings:

1. **App Name**: `Firezone`
1. **App logo**:
[Firezone logo](https://user-images.githubusercontent.com/52545545/155907625-a4f6c8c2-3952-488d-b244-3c37400846cf.png)
(save link as).
1. **Sign-in redirect URIs**: Append `/auth/okta/callback` to your Firezone base
URL. For example, if your Firezone instance is available at
`https://firezone.example.com`, then you would enter
`https://firezone.example.com/auth/okta/callback` here. The redirect URI is
where Okta will redirect the user's browser after successful authentication.
Firezone will receive this callback, initiate the user's session, and redirect
the user's browser to the appropriate page depending on the user's role.
1. **Assignments**:
Limit to the groups you wish to provide access to your Firezone instance.

![Okta Settings](https://user-images.githubusercontent.com/52545545/155907987-caa3318e-4871-488d-b1d4-deb397a17f19.png){:width="800"}

Once settings are saved, you will be given a Client ID, Client Secret, and Okta Domain.
These 3 values will be used in Step 2 to configure Firezone.

![Okta credentials](https://user-images.githubusercontent.com/52545545/156463942-7130b4bb-372a-4e27-ae06-7d3405214ec7.png){:width="800"}

## Step 2 - Configure Firezone

Using the client ID, secret, and redirect URI from above, edit the `/etc/firezone/firezone.rb`
configuration file to include the following options:

```ruby
# set the following variables to the values obtained in step 2
default['firezone']['authentication']['okta']['enabled'] = true
default['firezone']['authentication']['okta']['client_id'] = 'OKTA_CLIENT_ID'
default['firezone']['authentication']['okta']['client_secret'] = 'OKTA_CLIENT_SECRET'
default['firezone']['authentication']['okta']['site'] = 'OKTA_SITE'
```

Run the following commands to apply the changes:

```text
firezone-ctl reconfigure
firezone-ctl restart
```

You should now see a `Sign in with Okta` button at the root Firezone URL.
