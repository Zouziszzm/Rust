package com.opexence.dto;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;
import java.time.Instant;
import java.util.UUID;

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public record CreateExpenseRequest(
        UUID categoryId,
        UUID shopId,
        long amountTotal,
        @JsonProperty("subtotal_10_percent") Long subtotal10Percent,
        @JsonProperty("tax_amount_10_percent") Long taxAmount10Percent,
        @JsonProperty("subtotal_8_percent") Long subtotal8Percent,
        @JsonProperty("tax_amount_8_percent") Long taxAmount8Percent,
        Long taxExemptAmount,
        Long taxAmountTotal,
        String merchantName,
        String invoiceRegistrationNumber,
        Boolean isQualifiedInvoice,
        String receiptNumber,
        String title,
        String notes,
        Instant occurredAt,
        String paymentMethod,
        Boolean isRecurring,
        Boolean isRefundable
) {}
