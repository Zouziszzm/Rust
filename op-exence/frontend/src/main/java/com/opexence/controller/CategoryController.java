package com.opexence.controller;

import com.opexence.client.ApiClient;
import com.opexence.client.ApiException;
import com.opexence.dto.CategoryDto;
import com.opexence.dto.CreateCategoryRequest;
import com.opexence.dto.ExpenseSummaryResponse;
import com.opexence.dto.UpdateCategoryRequest;
import com.opexence.form.CategoryForm;
import com.opexence.util.YenFormatter;
import java.time.Instant;
import java.time.LocalDate;
import java.time.ZoneId;
import java.time.temporal.TemporalAdjusters;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.stream.Collectors;
import org.springframework.stereotype.Controller;
import org.springframework.ui.Model;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.ModelAttribute;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.servlet.mvc.support.RedirectAttributes;

@Controller
@RequestMapping("/categories")
public class CategoryController {

    private final ApiClient apiClient;

    public CategoryController(ApiClient apiClient) {
        this.apiClient = apiClient;
    }

    @ModelAttribute("categoryGroups")
    public List<String> categoryGroups() {
        return List.of(
                "housing", "parking", "groceries", "utilities", "transport", "dining",
                "healthcare", "insurance_tax", "shopping", "entertainment", "subscriptions",
                "personal_care", "car", "admin", "other");
    }

    @GetMapping
    public String list(Model model) {
        List<CategoryDto> categories = apiClient.listCategories(null).items();
        Map<String, List<CategoryDto>> grouped = categories.stream()
                .sorted(Comparator.comparing(
                                CategoryDto::categoryGroup,
                                Comparator.nullsLast(String::compareTo))
                        .thenComparing(CategoryDto::name, Comparator.nullsLast(String::compareTo)))
                .collect(Collectors.groupingBy(
                        c -> c.categoryGroup() != null ? c.categoryGroup() : "other",
                        LinkedHashMap::new,
                        Collectors.toList()));

        ZoneId zone = ZoneId.of("Asia/Tokyo");
        LocalDate now = LocalDate.now(zone);
        Instant from = now.with(TemporalAdjusters.firstDayOfMonth()).atStartOfDay(zone).toInstant();
        Instant to = now.plusDays(1).atStartOfDay(zone).toInstant();
        ExpenseSummaryResponse summary = apiClient.getSummary(from, to, null);

        Map<UUID, Long> categoryTotalsMap = summary.byCategory().stream()
                .collect(Collectors.toMap(
                        ct -> ct.categoryId(),
                        ct -> ct.total(),
                        (a, b) -> a));

        model.addAttribute("groupedCategories", grouped);
        model.addAttribute("categoryTotalsMap", categoryTotalsMap);
        model.addAttribute("totalCategories", categories.size());
        model.addAttribute("customCategories", categories.stream().filter(c -> !c.isSystem()).count());
        model.addAttribute("monthLabel", now.getMonth().toString() + " " + now.getYear());
        model.addAttribute("yen", YenFormatter.class);
        return "categories/list";
    }

    @GetMapping("/new")
    public String createForm(Model model) {
        CategoryForm form = new CategoryForm();
        form.setCategoryGroup("other");
        model.addAttribute("form", form);
        return "categories/form";
    }

    @PostMapping
    public String create(@ModelAttribute CategoryForm form, RedirectAttributes redirect) {
        try {
            apiClient.createCategory(new CreateCategoryRequest(
                    form.getSlug(), form.getName(), form.getCategoryGroup(), form.getDescription()));
            redirect.addFlashAttribute("successMessage", "Category created");
            return "redirect:/categories";
        } catch (ApiException ex) {
            DashboardController.flashApiError(redirect, ex);
            return "redirect:/categories/new";
        }
    }

    @GetMapping("/{id}/edit")
    public String editForm(@PathVariable UUID id, Model model) {
        CategoryDto category = apiClient.getCategory(id);
        if (category.isSystem()) {
            return "redirect:/categories";
        }
        CategoryForm form = new CategoryForm();
        form.setSlug(category.slug());
        form.setName(category.name());
        form.setCategoryGroup(category.categoryGroup());
        form.setDescription(category.description());
        model.addAttribute("form", form);
        model.addAttribute("categoryId", id);
        model.addAttribute("readOnlySlug", true);
        return "categories/form";
    }

    @PostMapping("/{id}")
    public String update(
            @PathVariable UUID id,
            @ModelAttribute CategoryForm form,
            RedirectAttributes redirect) {
        try {
            apiClient.updateCategory(id, new UpdateCategoryRequest(
                    form.getName(), form.getCategoryGroup(), form.getDescription()));
            redirect.addFlashAttribute("successMessage", "Category updated");
        } catch (ApiException ex) {
            DashboardController.flashApiError(redirect, ex);
        }
        return "redirect:/categories";
    }

    @PostMapping("/{id}/delete")
    public String delete(@PathVariable UUID id, RedirectAttributes redirect) {
        try {
            apiClient.deleteCategory(id);
            redirect.addFlashAttribute("successMessage", "Category deleted");
        } catch (ApiException ex) {
            DashboardController.flashApiError(redirect, ex);
        }
        return "redirect:/categories";
    }
}
