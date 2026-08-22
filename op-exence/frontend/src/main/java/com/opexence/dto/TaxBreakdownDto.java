package com.opexence.dto;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public record TaxBreakdownDto(
        @JsonProperty("tax_10_percent") long tax10Percent,
        @JsonProperty("tax_8_percent") long tax8Percent,
        long taxTotal,
        long salaryDeductionsTotal
) {}
