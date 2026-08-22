package com.opexence.dto;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;
import java.time.Instant;
import java.util.UUID;

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public record ExpenseDto(
        UUID id,
        UUID categoryId,
        UUID shopId,
        long amountTotal,
        @JsonProperty("subtotal_10_percent") long subtotal10Percent,
        @JsonProperty("tax_amount_10_percent") long taxAmount10Percent,
        @JsonProperty("subtotal_8_percent") long subtotal8Percent,
        @JsonProperty("tax_amount_8_percent") long taxAmount8Percent,
        long taxExemptAmount,
        long taxAmountTotal,
        String merchantName,
        String invoiceRegistrationNumber,
        boolean isQualifiedInvoice,
        String receiptNumber,
        String title,
        String notes,
        Instant occurredAt,
        String paymentMethod,
        boolean isRecurring,
        boolean isRefundable,
        Instant createdAt,
        Instant updatedAt,
        String categoryName,
        String categoryGroup,
        String shopName
) {}
