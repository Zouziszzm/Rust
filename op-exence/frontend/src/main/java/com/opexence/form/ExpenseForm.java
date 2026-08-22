package com.opexence.form;

import java.time.Instant;
import java.util.UUID;

public class ExpenseForm {

    private UUID id;
    private UUID categoryId;
    private UUID shopId;
    private Long amountTotal;
    private Long subtotal10Percent = 0L;
    private Long taxAmount10Percent = 0L;
    private Long subtotal8Percent = 0L;
    private Long taxAmount8Percent = 0L;
    private Long taxExemptAmount = 0L;
    private Long taxAmountTotal = 0L;
    private String merchantName;
    private String invoiceRegistrationNumber;
    private boolean isQualifiedInvoice = false;
    private String receiptNumber;
    private String title;
    private String notes;
    private Instant occurredAt = Instant.now();
    private String occurredAtInput;
    private String paymentMethod = "other";
    private boolean isRecurring = false;
    private boolean isRefundable = false;

    public UUID getId() {
        return id;
    }

    public void setId(UUID id) {
        this.id = id;
    }

    public UUID getCategoryId() {
        return categoryId;
    }

    public void setCategoryId(UUID categoryId) {
        this.categoryId = categoryId;
    }

    public UUID getShopId() {
        return shopId;
    }

    public void setShopId(UUID shopId) {
        this.shopId = shopId;
    }

    public Long getAmountTotal() {
        return amountTotal;
    }

    public void setAmountTotal(Long amountTotal) {
        this.amountTotal = amountTotal;
    }

    public Long getSubtotal10Percent() {
        return subtotal10Percent;
    }

    public void setSubtotal10Percent(Long subtotal10Percent) {
        this.subtotal10Percent = subtotal10Percent;
    }

    public Long getTaxAmount10Percent() {
        return taxAmount10Percent;
    }

    public void setTaxAmount10Percent(Long taxAmount10Percent) {
        this.taxAmount10Percent = taxAmount10Percent;
    }

    public Long getSubtotal8Percent() {
        return subtotal8Percent;
    }

    public void setSubtotal8Percent(Long subtotal8Percent) {
        this.subtotal8Percent = subtotal8Percent;
    }

    public Long getTaxAmount8Percent() {
        return taxAmount8Percent;
    }

    public void setTaxAmount8Percent(Long taxAmount8Percent) {
        this.taxAmount8Percent = taxAmount8Percent;
    }

    public Long getTaxExemptAmount() {
        return taxExemptAmount;
    }

    public void setTaxExemptAmount(Long taxExemptAmount) {
        this.taxExemptAmount = taxExemptAmount;
    }

    public Long getTaxAmountTotal() {
        return taxAmountTotal;
    }

    public void setTaxAmountTotal(Long taxAmountTotal) {
        this.taxAmountTotal = taxAmountTotal;
    }

    public String getMerchantName() {
        return merchantName;
    }

    public void setMerchantName(String merchantName) {
        this.merchantName = merchantName;
    }

    public String getInvoiceRegistrationNumber() {
        return invoiceRegistrationNumber;
    }

    public void setInvoiceRegistrationNumber(String invoiceRegistrationNumber) {
        this.invoiceRegistrationNumber = invoiceRegistrationNumber;
    }

    public boolean isQualifiedInvoice() {
        return isQualifiedInvoice;
    }

    public void setQualifiedInvoice(boolean qualifiedInvoice) {
        isQualifiedInvoice = qualifiedInvoice;
    }

    public String getReceiptNumber() {
        return receiptNumber;
    }

    public void setReceiptNumber(String receiptNumber) {
        this.receiptNumber = receiptNumber;
    }

    public String getTitle() {
        return title;
    }

    public void setTitle(String title) {
        this.title = title;
    }

    public String getNotes() {
        return notes;
    }

    public void setNotes(String notes) {
        this.notes = notes;
    }

    public Instant getOccurredAt() {
        return occurredAt;
    }

    public void setOccurredAt(Instant occurredAt) {
        this.occurredAt = occurredAt;
    }

    public String getOccurredAtInput() {
        return occurredAtInput;
    }

    public void setOccurredAtInput(String occurredAtInput) {
        this.occurredAtInput = occurredAtInput;
    }

    public String getPaymentMethod() {
        return paymentMethod;
    }

    public void setPaymentMethod(String paymentMethod) {
        this.paymentMethod = paymentMethod;
    }

    public boolean isRecurring() {
        return isRecurring;
    }

    public void setRecurring(boolean recurring) {
        isRecurring = recurring;
    }

    public boolean isRefundable() {
        return isRefundable;
    }

    public void setRefundable(boolean refundable) {
        isRefundable = refundable;
    }
}
