package com.opexence.controller;

import com.opexence.client.ApiClient;
import com.opexence.client.ApiException;
import com.opexence.dto.CategoryDto;
import com.opexence.dto.CreateExpenseRequest;
import com.opexence.dto.ExpenseDto;
import com.opexence.dto.ShopDto;
import com.opexence.dto.UpdateExpenseRequest;
import com.opexence.form.ExpenseForm;
import com.opexence.util.YenFormatter;
import java.beans.PropertyEditorSupport;
import java.time.Instant;
import java.time.LocalDate;
import java.time.ZoneId;
import java.time.format.DateTimeFormatter;
import java.util.List;
import java.util.UUID;
import org.springframework.web.bind.WebDataBinder;
import org.springframework.web.bind.annotation.InitBinder;
import org.springframework.stereotype.Controller;
import org.springframework.ui.Model;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.ModelAttribute;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.servlet.mvc.support.RedirectAttributes;

@Controller
@RequestMapping("/expenses")
public class ExpenseController {

    private static final ZoneId ZONE = ZoneId.of("Asia/Tokyo");
    private static final DateTimeFormatter DATE_FMT = DateTimeFormatter.ISO_LOCAL_DATE;

    private final ApiClient apiClient;

    public ExpenseController(ApiClient apiClient) {
        this.apiClient = apiClient;
    }

    @InitBinder
    public void initBinder(WebDataBinder binder) {
        binder.registerCustomEditor(UUID.class, new PropertyEditorSupport() {
            @Override
            public void setAsText(String text) {
                setValue(text == null || text.isBlank() ? null : UUID.fromString(text));
            }
        });
    }

    @ModelAttribute("paymentMethods")
    public List<String> paymentMethods() {
        return List.of(
                "cash", "credit_card", "debit_card", "paypay", "line_pay",
                "rakuten_pay", "suica_pasmo", "bank_transfer", "direct_debit", "other");
    }

    @GetMapping
    public String list(
            @RequestParam(required = false) UUID categoryId,
            @RequestParam(required = false) String group,
            @RequestParam(required = false) UUID shopId,
            @RequestParam(required = false) String paymentMethod,
            @RequestParam(required = false) Boolean recurring,
            @RequestParam(required = false) String from,
            @RequestParam(required = false) String to,
            Model model) {
        Instant fromInstant = parseDate(from);
        Instant toInstant = parseDateEnd(to);

        var response = apiClient.listExpenses(
                categoryId, group, shopId, paymentMethod, recurring,
                fromInstant, toInstant, 50, 0);

        model.addAttribute("expenses", response.items());
        model.addAttribute("categories", apiClient.listCategories(null).items());
        model.addAttribute("shops", apiClient.listShops().items());
        model.addAttribute("categoryId", categoryId);
        model.addAttribute("group", group);
        model.addAttribute("shopId", shopId);
        model.addAttribute("paymentMethod", paymentMethod);
        model.addAttribute("recurring", recurring);
        model.addAttribute("from", from);
        model.addAttribute("to", to);
        model.addAttribute("yen", YenFormatter.class);
        return "expenses/list";
    }

    @GetMapping("/new")
    public String createForm(Model model) {
        ExpenseForm form = new ExpenseForm();
        form.setOccurredAtInput(formatDate(Instant.now()));
        populateFormModel(model, form);
        return "expenses/form";
    }

    @PostMapping
    public String create(@ModelAttribute ExpenseForm form, RedirectAttributes redirect) {
        try {
            apiClient.createExpense(toCreateRequest(form));
            redirect.addFlashAttribute("successMessage", "Expense created");
            return "redirect:/expenses";
        } catch (ApiException ex) {
            DashboardController.flashApiError(redirect, ex);
            return "redirect:/expenses/new";
        }
    }

    @GetMapping("/{id}/edit")
    public String editForm(@PathVariable UUID id, Model model) {
        ExpenseDto expense = apiClient.getExpense(id);
        ExpenseForm form = fromDto(expense);
        model.addAttribute("expenseId", id);
        populateFormModel(model, form);
        return "expenses/form";
    }

    @PostMapping("/{id}")
    public String update(
            @PathVariable UUID id,
            @ModelAttribute ExpenseForm form,
            RedirectAttributes redirect) {
        try {
            apiClient.updateExpense(id, toUpdateRequest(form));
            redirect.addFlashAttribute("successMessage", "Expense updated");
            return "redirect:/expenses";
        } catch (ApiException ex) {
            DashboardController.flashApiError(redirect, ex);
            return "redirect:/expenses/" + id + "/edit";
        }
    }

    @PostMapping("/{id}/delete")
    public String delete(@PathVariable UUID id, RedirectAttributes redirect) {
        try {
            apiClient.deleteExpense(id);
            redirect.addFlashAttribute("successMessage", "Expense deleted");
        } catch (ApiException ex) {
            DashboardController.flashApiError(redirect, ex);
        }
        return "redirect:/expenses";
    }

    private void populateFormModel(Model model, ExpenseForm form) {
        model.addAttribute("form", form);
        model.addAttribute("categories", apiClient.listCategories(null).items());
        model.addAttribute("shops", apiClient.listShops().items());
        model.addAttribute("shopRequiredGroups", List.of("groceries", "personal_care"));
    }

    private ExpenseForm fromDto(ExpenseDto e) {
        ExpenseForm form = new ExpenseForm();
        form.setId(e.id());
        form.setCategoryId(e.categoryId());
        form.setShopId(e.shopId());
        form.setAmountTotal(e.amountTotal());
        form.setSubtotal10Percent(e.subtotal10Percent());
        form.setTaxAmount10Percent(e.taxAmount10Percent());
        form.setSubtotal8Percent(e.subtotal8Percent());
        form.setTaxAmount8Percent(e.taxAmount8Percent());
        form.setTaxExemptAmount(e.taxExemptAmount());
        form.setTaxAmountTotal(e.taxAmountTotal());
        form.setMerchantName(e.merchantName());
        form.setInvoiceRegistrationNumber(e.invoiceRegistrationNumber());
        form.setQualifiedInvoice(e.isQualifiedInvoice());
        form.setReceiptNumber(e.receiptNumber());
        form.setTitle(e.title());
        form.setNotes(e.notes());
        form.setOccurredAt(e.occurredAt());
        form.setOccurredAtInput(formatDate(e.occurredAt()));
        form.setPaymentMethod(e.paymentMethod());
        form.setRecurring(e.isRecurring());
        form.setRefundable(e.isRefundable());
        return form;
    }

    private CreateExpenseRequest toCreateRequest(ExpenseForm form) {
        Instant occurred = parseFormDate(form.getOccurredAtInput());
        return new CreateExpenseRequest(
                form.getCategoryId(),
                form.getShopId(),
                form.getAmountTotal(),
                form.getSubtotal10Percent(),
                form.getTaxAmount10Percent(),
                form.getSubtotal8Percent(),
                form.getTaxAmount8Percent(),
                form.getTaxExemptAmount(),
                form.getTaxAmountTotal(),
                form.getMerchantName(),
                form.getInvoiceRegistrationNumber(),
                form.isQualifiedInvoice(),
                form.getReceiptNumber(),
                form.getTitle(),
                form.getNotes(),
                occurred,
                form.getPaymentMethod(),
                form.isRecurring(),
                form.isRefundable());
    }

    private UpdateExpenseRequest toUpdateRequest(ExpenseForm form) {
        Instant occurred = parseFormDate(form.getOccurredAtInput());
        return new UpdateExpenseRequest(
                form.getCategoryId(),
                form.getShopId(),
                form.getAmountTotal(),
                form.getSubtotal10Percent(),
                form.getTaxAmount10Percent(),
                form.getSubtotal8Percent(),
                form.getTaxAmount8Percent(),
                form.getTaxExemptAmount(),
                form.getTaxAmountTotal(),
                form.getMerchantName(),
                form.getInvoiceRegistrationNumber(),
                form.isQualifiedInvoice(),
                form.getReceiptNumber(),
                form.getTitle(),
                form.getNotes(),
                occurred,
                form.getPaymentMethod(),
                form.isRecurring(),
                form.isRefundable());
    }

    private Instant parseFormDate(String input) {
        if (input == null || input.isBlank()) {
            return Instant.now();
        }
        return LocalDate.parse(input, DATE_FMT).atStartOfDay(ZONE).toInstant();
    }

    private String formatDate(Instant instant) {
        return LocalDate.ofInstant(instant, ZONE).format(DATE_FMT);
    }

    private Instant parseDate(String input) {
        if (input == null || input.isBlank()) return null;
        return LocalDate.parse(input, DATE_FMT).atStartOfDay(ZONE).toInstant();
    }

    private Instant parseDateEnd(String input) {
        if (input == null || input.isBlank()) return null;
        return LocalDate.parse(input, DATE_FMT).plusDays(1).atStartOfDay(ZONE).toInstant();
    }
}
