# Stripe Payment Links Setup Guide

## Converting Test Link to Live Payment Links

Your test link: `https://buy.stripe.com/test_7sY9ATc8t1GK0pj5j7enS00`

### ✅ Step 1: Access Stripe Dashboard
1. Go to: https://dashboard.stripe.com
2. Login with your Stripe account
3. Switch to **Live Mode** (toggle in top left corner)
4. Accept the live mode terms

### ✅ Step 2: Create Payment Links

Navigate to: **Payments** → **Payment Links**

#### Link 1: $3 Donation
1. Click "+ New"
2. **Product name**: `BNN Code - Supporter Donation`
3. **Amount**: $3.00 USD
4. **Description**: "Help keep BNN Code free and open-source"
5. **After payment**: Set to "Redirect to your website" → `https://github.com/Yotsawarit/bnn-code`
6. Click "Create link"
7. **Copy the link** (will look like: `https://buy.stripe.com/live_...`)

#### Link 2: $5 Donation
- Repeat steps 1-7
- **Product name**: `BNN Code - Contributor Donation`
- **Amount**: $5.00 USD
- **Description**: "Support development with priority issue response"

#### Link 3: $10 Donation
- Repeat steps 1-7
- **Product name**: `BNN Code - Maintainer Donation`
- **Amount**: $10.00 USD
- **Description**: "Support new feature development"

#### Link 4: $50 Donation
- Repeat steps 1-7
- **Product name**: `BNN Code - Patron Donation`
- **Amount**: $50.00 USD
- **Description**: "Exclusive support and feature requests"

### ✅ Step 3: Save Your Links

Create a `.env.example` file or document:
```
STRIPE_DONATION_3=https://buy.stripe.com/live_...
STRIPE_DONATION_5=https://buy.stripe.com/live_...
STRIPE_DONATION_10=https://buy.stripe.com/live_...
STRIPE_DONATION_50=https://buy.stripe.com/live_...
```

### ✅ Step 4: Test Your Links
- Click each link to verify they work
- Try using test card: `4242 4242 4242 4242` (Exp: any future date, CVC: any 3 digits)

### ✅ Step 5: Update README
We'll update the README.md with all your live payment links.

---

## Stripe Dashboard Features

Once set up, you can:
- **View Payments**: All transactions in real-time
- **Manage Refunds**: If needed
- **Download Reports**: For accounting
- **Set Up Webhooks**: For automated processing (optional)
- **Email Receipts**: Automatically sent to donors

---

## Important Notes

- Stripe charges ~2.9% + $0.30 per transaction
- Example: $3 donation = ~$2.21 to you
- Minimum transaction: $0.50 USD
- Payments typically arrive in 2-3 business days

---

## Support
- [Stripe Payment Links Help](https://stripe.com/docs/payments/payment-links)
- [Stripe Support](https://support.stripe.com)
