package com.opexence.client;

import com.fasterxml.jackson.databind.JsonNode;
import com.opexence.dto.CategoryDto;
import com.opexence.dto.CategoryListResponse;
import com.opexence.dto.CreateCategoryRequest;
import com.opexence.dto.CreateExpenseRequest;
import com.opexence.dto.ExpenseDto;
import com.opexence.dto.ExpenseListResponse;
import com.opexence.dto.ExpenseSummaryResponse;
import com.opexence.dto.MonthlySummaryResponse;
import com.opexence.dto.ShopDto;
import com.opexence.dto.ShopListResponse;
import com.opexence.dto.UpdateCategoryRequest;
import com.opexence.dto.UpdateExpenseRequest;
import java.time.Instant;
import java.util.UUID;
import org.springframework.http.HttpStatusCode;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;
import org.springframework.web.reactive.function.client.ClientResponse;
import org.springframework.web.reactive.function.client.WebClient;
import reactor.core.publisher.Mono;

@Component
public class ApiClient {

    private final WebClient webClient;

    public ApiClient(@Qualifier("apiWebClient") WebClient apiWebClient) {
        this.webClient = apiWebClient;
    }

    public CategoryListResponse listCategories(String group) {
        return webClient.get()
                .uri(uriBuilder -> {
                    var b = uriBuilder.path("/categories").queryParam("limit", 100);
                    if (group != null && !group.isBlank()) {
                        b.queryParam("group", group);
                    }
                    return b.build();
                })
                .retrieve()
                .onStatus(HttpStatusCode::isError, this::toApiException)
                .bodyToMono(CategoryListResponse.class)
                .block();
    }

    public CategoryDto getCategory(UUID id) {
        return webClient.get()
                .uri("/categories/{id}", id)
                .retrieve()
                .onStatus(HttpStatusCode::isError, this::toApiException)
                .bodyToMono(CategoryDto.class)
                .block();
    }

    public CategoryDto createCategory(CreateCategoryRequest request) {
        return webClient.post()
                .uri("/categories")
                .bodyValue(request)
                .retrieve()
                .onStatus(HttpStatusCode::isError, this::toApiException)
                .bodyToMono(CategoryDto.class)
                .block();
    }

    public CategoryDto updateCategory(UUID id, UpdateCategoryRequest request) {
        return webClient.patch()
                .uri("/categories/{id}", id)
                .bodyValue(request)
                .retrieve()
                .onStatus(HttpStatusCode::isError, this::toApiException)
                .bodyToMono(CategoryDto.class)
                .block();
    }

    public void deleteCategory(UUID id) {
        webClient.delete()
                .uri("/categories/{id}", id)
                .retrieve()
                .onStatus(HttpStatusCode::isError, this::toApiException)
                .toBodilessEntity()
                .block();
    }

    public ShopListResponse listShops() {
        return webClient.get()
                .uri("/shops")
                .retrieve()
                .onStatus(HttpStatusCode::isError, this::toApiException)
                .bodyToMono(ShopListResponse.class)
                .block();
    }

    public ShopDto getShop(UUID id) {
        return webClient.get()
                .uri("/shops/{id}", id)
                .retrieve()
                .onStatus(HttpStatusCode::isError, this::toApiException)
                .bodyToMono(ShopDto.class)
                .block();
    }

    public ExpenseListResponse listExpenses(
            UUID categoryId,
            String group,
            UUID shopId,
            String paymentMethod,
            Boolean recurring,
            Instant from,
            Instant to,
            long limit,
            long offset) {
        return webClient.get()
                .uri(uriBuilder -> {
                    var b = uriBuilder.path("/expenses")
                            .queryParam("limit", limit)
                            .queryParam("offset", offset);
                    if (categoryId != null) b.queryParam("category_id", categoryId);
                    if (group != null && !group.isBlank()) b.queryParam("group", group);
                    if (shopId != null) b.queryParam("shop_id", shopId);
                    if (paymentMethod != null && !paymentMethod.isBlank())
                        b.queryParam("payment_method", paymentMethod);
                    if (recurring != null) b.queryParam("is_recurring", recurring);
                    if (from != null) b.queryParam("from", from.toString());
                    if (to != null) b.queryParam("to", to.toString());
                    return b.build();
                })
                .retrieve()
                .onStatus(HttpStatusCode::isError, this::toApiException)
                .bodyToMono(ExpenseListResponse.class)
                .block();
    }

    public ExpenseDto getExpense(UUID id) {
        return webClient.get()
                .uri("/expenses/{id}", id)
                .retrieve()
                .onStatus(HttpStatusCode::isError, this::toApiException)
                .bodyToMono(ExpenseDto.class)
                .block();
    }

    public ExpenseDto createExpense(CreateExpenseRequest request) {
        return webClient.post()
                .uri("/expenses")
                .bodyValue(request)
                .retrieve()
                .onStatus(HttpStatusCode::isError, this::toApiException)
                .bodyToMono(ExpenseDto.class)
                .block();
    }

    public ExpenseDto updateExpense(UUID id, UpdateExpenseRequest request) {
        return webClient.patch()
                .uri("/expenses/{id}", id)
                .bodyValue(request)
                .retrieve()
                .onStatus(HttpStatusCode::isError, this::toApiException)
                .bodyToMono(ExpenseDto.class)
                .block();
    }

    public void deleteExpense(UUID id) {
        webClient.delete()
                .uri("/expenses/{id}", id)
                .retrieve()
                .onStatus(HttpStatusCode::isError, this::toApiException)
                .toBodilessEntity()
                .block();
    }

    public ExpenseSummaryResponse getSummary(Instant from, Instant to, String group) {
        return webClient.get()
                .uri(uriBuilder -> {
                    var b = uriBuilder.path("/expenses/summary");
                    if (from != null) b.queryParam("from", from.toString());
                    if (to != null) b.queryParam("to", to.toString());
                    if (group != null && !group.isBlank()) b.queryParam("group", group);
                    return b.build();
                })
                .retrieve()
                .onStatus(HttpStatusCode::isError, this::toApiException)
                .bodyToMono(ExpenseSummaryResponse.class)
                .block();
    }

    public MonthlySummaryResponse getMonthlySummary() {
        return webClient.get()
                .uri("/expenses/summary/monthly")
                .retrieve()
                .onStatus(HttpStatusCode::isError, this::toApiException)
                .bodyToMono(MonthlySummaryResponse.class)
                .block();
    }

    private Mono<? extends Throwable> toApiException(ClientResponse response) {
        return response.bodyToMono(JsonNode.class)
                .map(body -> {
                    String message =
                            body.has("error") ? body.get("error").asText() : "API request failed";
                    String code = body.has("code") ? body.get("code").asText() : "API_ERROR";
                    return new ApiException(message, code);
                })
                .defaultIfEmpty(new ApiException("API request failed", "API_ERROR"))
                .flatMap(Mono::error);
    }
}
